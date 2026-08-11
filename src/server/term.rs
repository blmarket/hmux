//! Resolved capabilities for one attached client terminal.
//!
//! Pane terminal state belongs to the hmux-vt emulator. This module describes the
//! outer terminal driven by the attach compositor: the terminfo values sent by
//! the tmux client, identify-time feature hints, and configured feature and
//! capability overrides.

use std::collections::BTreeMap;
use std::ffi::CString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CapabilityType {
    Flag,
    Number,
    String,
}

macro_rules! capability_catalog {
    ($($name:ident: $kind:ident,)*) => {
        /// Terminal capability codes, one per entry of tmux's tty-term.c
        /// catalog and in the same order: the ordinal is output-visible
        /// through [`ResolvedTerm::descriptions`]. Identify messages contain
        /// only these entries, and overrides for unknown names are ignored
        /// rather than creating ad-hoc capabilities.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[allow(non_camel_case_types)] // variants keep tmux's exact spellings
        #[repr(u8)]
        pub(crate) enum Capability {
            $($name,)*
        }

        impl Capability {
            const ALL: &'static [Capability] = &[$(Capability::$name),*];

            fn from_name(name: &str) -> Option<Self> {
                match name {
                    $(stringify!($name) => Some(Self::$name),)*
                    _ => None,
                }
            }

            fn name(self) -> &'static str {
                match self {
                    $(Self::$name => stringify!($name),)*
                }
            }

            fn value_type(self) -> CapabilityType {
                match self {
                    $(Self::$name => CapabilityType::$kind,)*
                }
            }
        }
    };
}

capability_catalog! {
    acsc: String,
    am: Flag,
    AX: Flag,
    bce: Flag,
    bel: String,
    Bidi: String,
    blink: String,
    bold: String,
    civis: String,
    clear: String,
    Clmg: String,
    Cmg: String,
    cnorm: String,
    colors: Number,
    Cr: String,
    Cs: String,
    csr: String,
    cub: String,
    cub1: String,
    cud: String,
    cud1: String,
    cuf: String,
    cuf1: String,
    cup: String,
    cuu: String,
    cuu1: String,
    cvvis: String,
    dch: String,
    dch1: String,
    dim: String,
    dl: String,
    dl1: String,
    Dsbp: String,
    Dseks: String,
    Dsfcs: String,
    Dsmg: String,
    E3: String,
    ech: String,
    ed: String,
    el: String,
    el1: String,
    enacs: String,
    Enbp: String,
    Eneks: String,
    Enfcs: String,
    Enmg: String,
    fsl: String,
    Hls: String,
    home: String,
    hpa: String,
    ich: String,
    ich1: String,
    il: String,
    il1: String,
    indn: String,
    invis: String,
    kcbt: String,
    kcub1: String,
    kcud1: String,
    kcuf1: String,
    kcuu1: String,
    kDC: String,
    kDC3: String,
    kDC4: String,
    kDC5: String,
    kDC6: String,
    kDC7: String,
    kdch1: String,
    kDN: String,
    kDN3: String,
    kDN4: String,
    kDN5: String,
    kDN6: String,
    kDN7: String,
    kend: String,
    kEND: String,
    kEND3: String,
    kEND4: String,
    kEND5: String,
    kEND6: String,
    kEND7: String,
    kf1: String,
    kf10: String,
    kf11: String,
    kf12: String,
    kf13: String,
    kf14: String,
    kf15: String,
    kf16: String,
    kf17: String,
    kf18: String,
    kf19: String,
    kf2: String,
    kf20: String,
    kf21: String,
    kf22: String,
    kf23: String,
    kf24: String,
    kf25: String,
    kf26: String,
    kf27: String,
    kf28: String,
    kf29: String,
    kf3: String,
    kf30: String,
    kf31: String,
    kf32: String,
    kf33: String,
    kf34: String,
    kf35: String,
    kf36: String,
    kf37: String,
    kf38: String,
    kf39: String,
    kf4: String,
    kf40: String,
    kf41: String,
    kf42: String,
    kf43: String,
    kf44: String,
    kf45: String,
    kf46: String,
    kf47: String,
    kf48: String,
    kf49: String,
    kf5: String,
    kf50: String,
    kf51: String,
    kf52: String,
    kf53: String,
    kf54: String,
    kf55: String,
    kf56: String,
    kf57: String,
    kf58: String,
    kf59: String,
    kf6: String,
    kf60: String,
    kf61: String,
    kf62: String,
    kf63: String,
    kf7: String,
    kf8: String,
    kf9: String,
    kHOM: String,
    kHOM3: String,
    kHOM4: String,
    kHOM5: String,
    kHOM6: String,
    kHOM7: String,
    khome: String,
    kIC: String,
    kIC3: String,
    kIC4: String,
    kIC5: String,
    kIC6: String,
    kIC7: String,
    kich1: String,
    kind: String,
    kLFT: String,
    kLFT3: String,
    kLFT4: String,
    kLFT5: String,
    kLFT6: String,
    kLFT7: String,
    kmous: String,
    knp: String,
    kNXT: String,
    kNXT3: String,
    kNXT4: String,
    kNXT5: String,
    kNXT6: String,
    kNXT7: String,
    kpp: String,
    kPRV: String,
    kPRV3: String,
    kPRV4: String,
    kPRV5: String,
    kPRV6: String,
    kPRV7: String,
    kri: String,
    kRIT: String,
    kRIT3: String,
    kRIT4: String,
    kRIT5: String,
    kRIT6: String,
    kRIT7: String,
    kUP: String,
    kUP3: String,
    kUP4: String,
    kUP5: String,
    kUP6: String,
    kUP7: String,
    Ms: String,
    Nobr: String,
    ol: String,
    op: String,
    Rect: String,
    rev: String,
    RGB: Flag,
    ri: String,
    rin: String,
    rmacs: String,
    rmcup: String,
    rmkx: String,
    Se: String,
    setab: String,
    setaf: String,
    setal: String,
    setrgbb: String,
    setrgbf: String,
    Setulc: String,
    Setulc1: String,
    sgr0: String,
    sitm: String,
    smacs: String,
    smcup: String,
    smkx: String,
    Smol: String,
    smso: String,
    smul: String,
    Smulx: String,
    smxx: String,
    Spb: String,
    Sxl: Flag,
    Ss: String,
    Swd: String,
    Sync: String,
    Tc: Flag,
    tsl: String,
    U8: Number,
    vpa: String,
    XT: Flag,
}

const DEFAULT_TERMINAL_FEATURES: &[&str] = &[
    "xterm*:clipboard:ccolour:cstyle:focus:title",
    "screen*:title",
    "rxvt*:ignorefkeys",
];
const DEFAULT_TERMINAL_OVERRIDES: &[&str] = &["linux*:AX@"];

const TERM_256_COLOURS: u8 = 0x01;
const TERM_NO_AM: u8 = 0x02;
const TERM_DECSLRM: u8 = 0x04;
const TERM_DECFRA: u8 = 0x08;
const TERM_RGB_COLOURS: u8 = 0x10;
const TERM_VT100_LIKE: u8 = 0x20;
const TERM_SIXEL: u8 = 0x40;

#[derive(Clone, Copy)]
struct Feature {
    name: &'static str,
    capabilities: &'static [&'static str],
    flags: u8,
}

const FEATURE_256: &[&str] = &[
    "AX",
    "setab=\x1b[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m",
    "setaf=\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m",
];
const FEATURE_BPASTE: &[&str] = &["Enbp=\x1b[?2004h", "Dsbp=\x1b[?2004l"];
const FEATURE_CCOLOUR: &[&str] = &["Cs=\x1b]12;%p1%s\x07", "Cr=\x1b]112\x07"];
const FEATURE_CLIPBOARD: &[&str] = &["Ms=\x1b]52;%p1%s;%p2%s\x07"];
const FEATURE_HYPERLINKS: &[&str] = &["Hls=\x1b]8;%?%p1%l%tid=%p1%s%;;%p2%s\x1b\\"];
const FEATURE_CSTYLE: &[&str] = &["Ss=\x1b[%p1%d q", "Se=\x1b[2 q"];
const FEATURE_EXTKEYS: &[&str] = &["Eneks=\x1b[>4;2m", "Dseks=\x1b[>4m"];
const FEATURE_FOCUS: &[&str] = &["Enfcs=\x1b[?1004h", "Dsfcs=\x1b[?1004l"];
const FEATURE_IGNORE_FKEYS: &[&str] = &[
    "kf0@", "kf1@", "kf2@", "kf3@", "kf4@", "kf5@", "kf6@", "kf7@", "kf8@", "kf9@", "kf10@",
    "kf11@", "kf12@", "kf13@", "kf14@", "kf15@", "kf16@", "kf17@", "kf18@", "kf19@", "kf20@",
    "kf21@", "kf22@", "kf23@", "kf24@", "kf25@", "kf26@", "kf27@", "kf28@", "kf29@", "kf30@",
    "kf31@", "kf32@", "kf33@", "kf34@", "kf35@", "kf36@", "kf37@", "kf38@", "kf39@", "kf40@",
    "kf41@", "kf42@", "kf43@", "kf44@", "kf45@", "kf46@", "kf47@", "kf48@", "kf49@", "kf50@",
    "kf51@", "kf52@", "kf53@", "kf54@", "kf55@", "kf56@", "kf57@", "kf58@", "kf59@", "kf60@",
    "kf61@", "kf62@", "kf63@",
];
const FEATURE_MARGINS: &[&str] = &[
    "Enmg=\x1b[?69h",
    "Dsmg=\x1b[?69l",
    "Clmg=\x1b[s",
    "Cmg=\x1b[%i%p1%d;%p2%ds",
];
const FEATURE_MOUSE: &[&str] = &["kmous=\x1b[M"];
const FEATURE_OSC7: &[&str] = &["Swd=\x1b]7;", "fsl=\x07"];
const FEATURE_OVERLINE: &[&str] = &["Smol=\x1b[53m"];
const FEATURE_PROGRESSBAR: &[&str] = &["Spb=\x1b]9;4;%p1%d;%p2%d\x1b\\"];
const FEATURE_RECTFILL: &[&str] = &["Rect"];
const FEATURE_RGB: &[&str] = &[
    "AX",
    "setrgbf=\x1b[38;2;%p1%d;%p2%d;%p3%dm",
    "setrgbb=\x1b[48;2;%p1%d;%p2%d;%p3%dm",
    "setab=\x1b[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m",
    "setaf=\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m",
];
const FEATURE_SIXEL: &[&str] = &["Sxl"];
const FEATURE_STRIKETHROUGH: &[&str] = &["smxx=\x1b[9m"];
const FEATURE_SYNC: &[&str] = &["Sync=\x1b[?2026%?%p1%{1}%-%tl%eh%;"];
const FEATURE_TITLE: &[&str] = &["tsl=\x1b]0;", "fsl=\x07"];
const FEATURE_USSTYLE: &[&str] = &[
    "Smulx=\x1b[4::%p1%dm",
    "Setulc=\x1b[58::2::%p1%{65536}%/%d::%p1%{256}%/%{255}%&%d::%p1%{255}%&%d%;m",
    "Setulc1=\x1b[58::5::%p1%dm",
    "ol=\x1b[59m",
];

// The index is part of the identify wire protocol (MSG_IDENTIFY_FEATURES).
const FEATURES: &[Feature] = &[
    Feature {
        name: "256",
        capabilities: FEATURE_256,
        flags: TERM_256_COLOURS,
    },
    Feature {
        name: "bpaste",
        capabilities: FEATURE_BPASTE,
        flags: 0,
    },
    Feature {
        name: "ccolour",
        capabilities: FEATURE_CCOLOUR,
        flags: 0,
    },
    Feature {
        name: "clipboard",
        capabilities: FEATURE_CLIPBOARD,
        flags: 0,
    },
    Feature {
        name: "hyperlinks",
        capabilities: FEATURE_HYPERLINKS,
        flags: 0,
    },
    Feature {
        name: "cstyle",
        capabilities: FEATURE_CSTYLE,
        flags: 0,
    },
    Feature {
        name: "extkeys",
        capabilities: FEATURE_EXTKEYS,
        flags: 0,
    },
    Feature {
        name: "focus",
        capabilities: FEATURE_FOCUS,
        flags: 0,
    },
    Feature {
        name: "ignorefkeys",
        capabilities: FEATURE_IGNORE_FKEYS,
        flags: 0,
    },
    Feature {
        name: "margins",
        capabilities: FEATURE_MARGINS,
        flags: TERM_DECSLRM,
    },
    Feature {
        name: "mouse",
        capabilities: FEATURE_MOUSE,
        flags: 0,
    },
    Feature {
        name: "osc7",
        capabilities: FEATURE_OSC7,
        flags: 0,
    },
    Feature {
        name: "overline",
        capabilities: FEATURE_OVERLINE,
        flags: 0,
    },
    Feature {
        name: "progressbar",
        capabilities: FEATURE_PROGRESSBAR,
        flags: 0,
    },
    Feature {
        name: "rectfill",
        capabilities: FEATURE_RECTFILL,
        flags: TERM_DECFRA,
    },
    Feature {
        name: "RGB",
        capabilities: FEATURE_RGB,
        flags: TERM_256_COLOURS | TERM_RGB_COLOURS,
    },
    // tmux's `tty_feature_sixel` is in the feature table whether or not the
    // build renders sixel, so `terminal-features` still names it and
    // `#{client_termfeatures}` still reports it. Nothing reads `TERM_SIXEL`:
    // the pinned oracle is built `--disable-sixel` and hmux draws no images.
    Feature {
        name: "sixel",
        capabilities: FEATURE_SIXEL,
        flags: TERM_SIXEL,
    },
    Feature {
        name: "strikethrough",
        capabilities: FEATURE_STRIKETHROUGH,
        flags: 0,
    },
    Feature {
        name: "sync",
        capabilities: FEATURE_SYNC,
        flags: 0,
    },
    Feature {
        name: "title",
        capabilities: FEATURE_TITLE,
        flags: 0,
    },
    Feature {
        name: "usstyle",
        capabilities: FEATURE_USSTYLE,
        flags: 0,
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityValue {
    Flag(bool),
    Number(i32),
    String(Vec<u8>),
}

/// Read-only capability boundary used by terminal input and output consumers.
/// There is one production implementation, [`ResolvedTerm`].
pub(crate) trait TerminalCapabilities {
    fn name(&self) -> &str;

    fn capability(&self, capability: Capability) -> Option<&CapabilityValue>;

    #[allow(dead_code)] // consumed as terminal-feature-driven output migrates here
    fn has_feature(&self, name: &str) -> bool;

    /// Whether the client's terminal is UTF-8 capable, as tmux's
    /// `CLIENT_UTF8` client flag reports it.
    fn utf8(&self) -> bool;

    fn generation(&self) -> u64;

    fn flag(&self, capability: Capability) -> bool {
        matches!(
            self.capability(capability),
            Some(CapabilityValue::Flag(true))
        )
    }

    fn auto_margin(&self) -> bool {
        self.flag(Capability::am)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)] // string parameters are used as clipboard/title output migrates here
pub(crate) enum CapabilityParameter<'a> {
    Number(i32),
    String(&'a str),
}

pub(crate) fn string_capability<'a>(
    terminal: &'a dyn TerminalCapabilities,
    capability: Capability,
) -> Option<&'a [u8]> {
    match terminal.capability(capability) {
        Some(CapabilityValue::String(value)) => Some(value),
        _ => None,
    }
}

#[allow(dead_code)] // consumed as color selection migrates to resolved capabilities
pub(crate) fn number_capability(
    terminal: &dyn TerminalCapabilities,
    capability: Capability,
) -> Option<i32> {
    match terminal.capability(capability) {
        Some(CapabilityValue::Number(value)) => Some(*value),
        _ => None,
    }
}

/// Expand a terminfo string with the same numeric and string parameter forms
/// exposed by tmux's tty_term_string_* family.
///
/// The expansion is performed by the pure-Rust `terminfo::expand` engine rather
/// than the C `tparm`; capability strings arrive from the tmux client identify
/// message, so only the parameter interpreter is needed, not the terminfo
/// database.
///
/// This engine is stricter than ncurses `tparm` about malformed format strings:
/// ncurses silently ignores a `%;` that closes no open `%?`, whereas the crate
/// rejects the whole string. Normalize those unmatched terminators before
/// expansion so the terminal adapter has tmux's leniency without weakening the
/// parser for balanced conditionals.
pub(crate) fn expand_capability(
    terminal: &dyn TerminalCapabilities,
    capability: Capability,
    parameters: &[CapabilityParameter<'_>],
) -> Option<Vec<u8>> {
    use terminfo::expand::{Context, Expand, Parameter};

    let value = string_capability(terminal, capability)?;
    let arguments = parameters
        .iter()
        .map(|parameter| match parameter {
            CapabilityParameter::Number(value) => Parameter::Number(*value),
            CapabilityParameter::String(value) => Parameter::String(value.as_bytes().to_vec()),
        })
        .collect::<Vec<_>>();

    let mut context = Context::default();
    let normalized = strip_unmatched_conditional_ends(value);
    let mut expanded = Vec::new();
    normalized
        .as_slice()
        .expand(&mut expanded, &arguments, &mut context)
        .ok()?;
    Some(expanded)
}

/// Remove only conditional terminators that have no matching `%?` opener.
///
/// ncurses treats those stray `%;` sequences as no-ops. Keeping balanced
/// conditionals intact lets the stricter Rust expander continue to validate
/// the rest of the capability string.
fn strip_unmatched_conditional_ends(value: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(value.len());
    let mut conditional_depth = 0usize;
    let mut index = 0;
    while index < value.len() {
        if value[index] == b'%' {
            if let Some(operator) = value.get(index + 1).copied() {
                match operator {
                    b'%' => {
                        normalized.extend_from_slice(&value[index..=index + 1]);
                        index += 2;
                        continue;
                    }
                    b'?' => {
                        conditional_depth += 1;
                        normalized.extend_from_slice(&value[index..=index + 1]);
                        index += 2;
                        continue;
                    }
                    b';' if conditional_depth == 0 => {
                        index += 2;
                        continue;
                    }
                    b';' => {
                        conditional_depth -= 1;
                        normalized.extend_from_slice(&value[index..=index + 1]);
                        index += 2;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        normalized.push(value[index]);
        index += 1;
    }
    normalized
}

/// Immutable identify data retained so configured features and overrides can
/// be re-resolved when a command client changes them after attach.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TerminalIdentity {
    name: String,
    capabilities: Vec<String>,
    feature_bits: u32,
    colorterm: Option<String>,
    utf8: bool,
}

impl TerminalIdentity {
    pub(crate) fn new(
        name: impl Into<String>,
        capabilities: Vec<String>,
        feature_bits: u32,
        colorterm: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            capabilities,
            feature_bits,
            colorterm,
            utf8: false,
        }
    }

    pub(crate) fn with_utf8(mut self, utf8: bool) -> Self {
        self.utf8 = utf8;
        self
    }
}

/// One attached client's effective terminal profile.
///
/// Capabilities are stored by [`Capability`] ordinal; features are the bitmask
/// whose bit positions are `FEATURES` table indices, the same encoding the
/// identify wire protocol uses.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ResolvedTerm {
    identity: TerminalIdentity,
    capabilities: Vec<Option<CapabilityValue>>,
    features: u32,
    terminal_flags: u8,
    acs: BTreeMap<u8, u8>,
    generation: u64,
}

fn feature_bit(name: &str) -> Option<u32> {
    FEATURES
        .iter()
        .position(|feature| feature.name == name)
        .map(|index| 1 << index)
}

impl ResolvedTerm {
    pub(crate) fn resolve<'a>(
        identity: TerminalIdentity,
        options: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        let mut resolved = Self {
            identity,
            ..Self::default()
        };
        resolved.refresh(options);
        resolved
    }

    pub(crate) fn refresh<'a>(&mut self, options: impl IntoIterator<Item = (&'a str, &'a str)>) {
        self.generation = self.generation.wrapping_add(1);
        self.capabilities.clear();
        self.capabilities.resize(Capability::ALL.len(), None);
        for entry in &self.identity.capabilities {
            let Some((name, value)) = entry.split_once('=') else {
                continue;
            };
            let Some(capability) = Capability::from_name(name) else {
                continue;
            };
            if let Some(value) = identify_value(capability, value) {
                self.capabilities[capability as usize] = Some(value);
            }
        }

        const KNOWN_FEATURES: u32 = (1 << FEATURES.len()) - 1;
        self.features = self.identity.feature_bits & KNOWN_FEATURES;
        match self.identity.colorterm.as_deref() {
            Some(value)
                if value.eq_ignore_ascii_case("truecolor")
                    || value.eq_ignore_ascii_case("24bit") =>
            {
                self.add_features(&["RGB"]);
            }
            Some(value) if value.contains("256") => {
                self.add_features(&["256"]);
            }
            _ => {}
        }

        let options = terminal_option_values(options);
        for value in options
            .iter()
            .filter(|value| value.name == "terminal-features")
            .flat_map(|value| split_array_entries(value.value))
        {
            let mut fields = split_override_fields(value).into_iter();
            let Some(pattern) = fields.next() else {
                continue;
            };
            if !terminal_pattern_matches(&pattern, &self.identity.name) {
                continue;
            }
            for feature in fields {
                let Some(index) = FEATURES
                    .iter()
                    .position(|known| known.name.eq_ignore_ascii_case(&feature))
                else {
                    // tty_add_features stops at the first unknown name.
                    break;
                };
                self.features |= 1 << index;
            }
        }

        // tmux applies overrides once before inference, then reapplies them
        // after feature capabilities so user configuration always wins.
        self.apply_overrides(&options);
        self.recompute_derived_flags(0);

        let vt100_like = self.flag_value(Capability::XT)
            || self
                .string_value(Capability::clear)
                .is_some_and(|clear| clear.starts_with(b"\x1b["));
        if vt100_like {
            self.add_features(&["bpaste", "focus", "title"]);
        }

        if (self.flag_value(Capability::Tc) || self.has_capability(Capability::RGB))
            && (!self.has_capability(Capability::setrgbf)
                || !self.has_capability(Capability::setrgbb))
        {
            self.add_features(&["RGB"]);
        }

        let mut feature_flags = 0;
        for (index, feature) in FEATURES.iter().enumerate() {
            if self.features & (1 << index) == 0 {
                continue;
            }
            for capabilities in feature.capabilities {
                self.apply_capabilities(capabilities);
            }
            feature_flags |= feature.flags;
        }

        // tmux's nested screen-family clients use ECMA-48 overline even when
        // the base terminfo entry predates the extended Smol name. Resolve it
        // into the profile before the final override pass so `Smol@` can still
        // explicitly remove it.
        if (self.identity.name.starts_with("screen") || self.identity.name.starts_with("tmux"))
            && !self.has_capability(Capability::Smol)
        {
            self.apply_capabilities(FEATURE_OVERLINE[0]);
            self.add_features(&["overline"]);
        }

        self.apply_overrides(&options);
        if vt100_like {
            feature_flags |= TERM_VT100_LIKE;
        }
        self.recompute_derived_flags(feature_flags);
        self.rebuild_acs();
    }

    pub(crate) fn validation_error(&self) -> Option<&'static str> {
        if !self.has_capability(Capability::clear) {
            Some("terminal does not support clear")
        } else if !self.has_capability(Capability::cup) {
            Some("terminal does not support cup")
        } else {
            None
        }
    }

    pub(crate) fn flags(&self) -> u8 {
        self.terminal_flags
    }

    /// Whether the terminal answers the VT100-family private sequences tmux
    /// sends unconditionally (theme subscription, extended keys).
    pub(crate) fn is_vt100_like(&self) -> bool {
        self.terminal_flags & TERM_VT100_LIKE != 0
    }

    /// The resolved features in tmux's feature-bit order, as
    /// `#{client_termfeatures}` reports them (`tty_get_features`).
    pub(crate) fn feature_list(&self) -> String {
        FEATURES
            .iter()
            .enumerate()
            .filter(|(index, _)| self.features & (1 << index) != 0)
            .map(|(_, feature)| feature.name)
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn descriptions(&self) -> Vec<String> {
        Capability::ALL
            .iter()
            .map(|capability| {
                let index = *capability as usize;
                let description = match self.get(*capability) {
                    None => "[missing]".to_string(),
                    Some(CapabilityValue::Flag(value)) => format!("(flag) {value}"),
                    Some(CapabilityValue::Number(value)) => format!("(number) {value}"),
                    Some(CapabilityValue::String(value)) => {
                        format!("(string) {}", visible_capability(value))
                    }
                };
                format!("{index:4}: {}: {description}", capability.name())
            })
            .collect()
    }

    fn add_features(&mut self, names: &[&str]) {
        for name in names {
            self.features |= feature_bit(name).expect("catalogued feature name");
        }
    }

    fn apply_overrides(&mut self, options: &[TerminalOptionValue<'_>]) {
        for value in options
            .iter()
            .filter(|value| value.name == "terminal-overrides")
            .flat_map(|value| split_array_entries(value.value))
        {
            self.apply_override(value);
        }
    }

    fn apply_override(&mut self, value: &str) {
        let mut fields = split_override_fields(value).into_iter();
        let Some(pattern) = fields.next() else {
            return;
        };
        if !terminal_pattern_matches(&pattern, &self.identity.name) {
            return;
        }
        for field in fields {
            self.apply_capability_field(&field);
        }
    }

    fn apply_capabilities(&mut self, capabilities: &str) {
        for field in split_override_fields(capabilities) {
            self.apply_capability_field(&field);
        }
    }

    fn apply_capability_field(&mut self, field: &str) {
        if field.is_empty() {
            return;
        }
        if let Some(name) = field.strip_suffix('@').filter(|_| !field.contains('=')) {
            if let Some(capability) = Capability::from_name(name) {
                self.capabilities[capability as usize] = None;
            }
            return;
        }

        let (name, encoded) = field.split_once('=').unwrap_or((field, ""));
        let Some(capability) = Capability::from_name(name) else {
            return;
        };
        let decoded = decode_vis(encoded).unwrap_or_else(|| encoded.as_bytes().to_vec());
        let value = match capability.value_type() {
            // The presence of a flag in an override enables it. tmux ignores
            // the assigned value even for spellings such as `am=0`.
            CapabilityType::Flag => CapabilityValue::Flag(true),
            CapabilityType::Number => {
                let Ok(value) = std::str::from_utf8(&decoded).unwrap_or("").parse::<i32>() else {
                    return;
                };
                if value < 0 {
                    return;
                }
                CapabilityValue::Number(value)
            }
            CapabilityType::String => CapabilityValue::String(decoded),
        };
        self.capabilities[capability as usize] = Some(value);
    }

    fn get(&self, capability: Capability) -> Option<&CapabilityValue> {
        self.capabilities
            .get(capability as usize)
            .and_then(Option::as_ref)
    }

    fn has_capability(&self, capability: Capability) -> bool {
        self.get(capability).is_some()
    }

    fn flag_value(&self, capability: Capability) -> bool {
        matches!(self.get(capability), Some(CapabilityValue::Flag(true)))
    }

    fn string_value(&self, capability: Capability) -> Option<&[u8]> {
        match self.get(capability) {
            Some(CapabilityValue::String(value)) => Some(value),
            _ => None,
        }
    }

    fn recompute_derived_flags(&mut self, base: u8) {
        self.terminal_flags = base;
        if self.has_capability(Capability::setrgbf) && self.has_capability(Capability::setrgbb) {
            self.terminal_flags |= TERM_RGB_COLOURS;
        } else {
            self.terminal_flags &= !TERM_RGB_COLOURS;
        }
        if self.has_capability(Capability::Cmg) && self.has_capability(Capability::Clmg) {
            self.terminal_flags |= TERM_DECSLRM;
        } else {
            self.terminal_flags &= !TERM_DECSLRM;
        }
        if self.has_capability(Capability::Rect) {
            self.terminal_flags |= TERM_DECFRA;
        } else {
            self.terminal_flags &= !TERM_DECFRA;
        }
        if !self.flag_value(Capability::am) {
            self.terminal_flags |= TERM_NO_AM;
        } else {
            self.terminal_flags &= !TERM_NO_AM;
        }
    }

    fn rebuild_acs(&mut self) {
        const ASCII_ACS: &str = "a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.";
        self.acs.clear();
        let acs = self
            .string_value(Capability::acsc)
            .unwrap_or(ASCII_ACS.as_bytes())
            .to_vec();
        for pair in acs.chunks_exact(2) {
            self.acs.insert(pair[0], pair[1]);
        }
    }

    #[cfg(test)]
    fn acs(&self, input: u8) -> Option<u8> {
        self.acs.get(&input).copied()
    }
}

fn visible_capability(value: &[u8]) -> String {
    let mut output = String::new();
    for byte in value {
        let visible = match byte {
            b'\\' => "\\\\".to_string(),
            b'\n' => "\\n".to_string(),
            b'\r' => "\\r".to_string(),
            b'\t' => "\\t".to_string(),
            0x07 => "\\a".to_string(),
            0x08 => "\\b".to_string(),
            0x0b => "\\v".to_string(),
            0x0c => "\\f".to_string(),
            0x1b => "\\033".to_string(),
            0x20..=0x7e => char::from(*byte).to_string(),
            byte => format!("\\{byte:03o}"),
        };
        if output.len() + visible.len() > 127 {
            break;
        }
        output.push_str(&visible);
    }
    output
}

impl TerminalCapabilities for ResolvedTerm {
    fn name(&self) -> &str {
        &self.identity.name
    }

    fn capability(&self, capability: Capability) -> Option<&CapabilityValue> {
        self.get(capability)
    }

    fn utf8(&self) -> bool {
        self.identity.utf8
    }

    fn has_feature(&self, name: &str) -> bool {
        let Some(index) = FEATURES.iter().position(|feature| feature.name == name) else {
            return false;
        };
        if self.features & (1 << index) != 0 {
            return true;
        }
        let feature = &FEATURES[index];
        if name == "ignorefkeys" || self.terminal_flags & feature.flags != feature.flags {
            return false;
        }
        feature.capabilities.iter().all(|capability| {
            let name = capability
                .split_once('=')
                .map_or(*capability, |(name, _)| name);
            let name = name.strip_suffix('@').unwrap_or(name);
            Capability::from_name(name).is_some_and(|capability| self.has_capability(capability))
        })
    }

    fn generation(&self) -> u64 {
        self.generation
    }
}

pub(crate) fn terminal_acs(terminal: &dyn TerminalCapabilities, input: u8) -> Option<u8> {
    const ASCII_ACS: &[u8] = b"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.";
    let acs = match terminal.capability(Capability::acsc) {
        Some(CapabilityValue::String(value)) => value.as_slice(),
        _ => ASCII_ACS,
    };
    acs.chunks_exact(2)
        .find(|pair| pair[0] == input)
        .map(|pair| pair[1])
}

/// Width that may be written at `row` without triggering the no-auto-margin
/// bottom-right workaround. Logical rows remain full width.
pub(crate) fn writable_width(
    terminal: &dyn TerminalCapabilities,
    row: u16,
    columns: u16,
    rows: u16,
) -> usize {
    let width = if !terminal.auto_margin() && row == rows {
        columns.saturating_sub(1)
    } else {
        columns
    };
    usize::from(width)
}

fn identify_value(capability: Capability, value: &str) -> Option<CapabilityValue> {
    match capability.value_type() {
        CapabilityType::Flag => Some(CapabilityValue::Flag(value == "1")),
        CapabilityType::Number => value
            .parse::<i32>()
            .ok()
            .filter(|value| *value >= 0)
            .map(CapabilityValue::Number),
        CapabilityType::String => Some(CapabilityValue::String(strip_padding(value))),
    }
}

/// Remove terminfo delay markers. tmux emits terminal output without honoring
/// padding, so `$<5>` and `$<5/>` never reach the outer terminal.
fn strip_padding(value: &str) -> Vec<u8> {
    if !value.contains('$') {
        return value.as_bytes().to_vec();
    }
    let bytes = value.as_bytes();
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'$' && bytes.get(index + 1) == Some(&b'<') {
            let Some(end) = bytes[index + 2..].iter().position(|byte| *byte == b'>') else {
                break;
            };
            index += end + 3;
            continue;
        }
        stripped.push(bytes[index]);
        index += 1;
    }
    stripped
}

/// Decode the vis(3) spellings accepted by tmux terminal overrides. Returning
/// `None` preserves the original value, matching tmux's strunvis fallback.
fn decode_vis(value: &str) -> Option<Vec<u8>> {
    let bytes = value
        .as_bytes()
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'\\' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }
        index += 1;
        let escaped = *bytes.get(index)?;
        index += 1;
        match escaped {
            b'\\' => decoded.push(b'\\'),
            b'E' => decoded.push(0x1b),
            b'n' => decoded.push(b'\n'),
            b'r' => decoded.push(b'\r'),
            b'b' => decoded.push(0x08),
            b'a' => decoded.push(0x07),
            b'v' => decoded.push(0x0b),
            b't' => decoded.push(b'\t'),
            b'f' => decoded.push(0x0c),
            b's' => decoded.push(b' '),
            b'\n' | b'$' => {}
            b'M' => {
                let kind = *bytes.get(index)?;
                index += 1;
                let value = *bytes.get(index)?;
                index += 1;
                decoded.push(match kind {
                    b'-' => 0x80 | value,
                    b'^' if value == b'?' => 0xff,
                    b'^' => 0x80 | (value & 0x1f),
                    _ => return None,
                });
            }
            b'^' => {
                let control = *bytes.get(index)?;
                index += 1;
                decoded.push(if control == b'?' {
                    0x7f
                } else {
                    control & 0x1f
                });
            }
            b'0'..=b'7' => {
                let mut byte = escaped - b'0';
                let mut digits = 1;
                while digits < 3 && index < bytes.len() && matches!(bytes[index], b'0'..=b'7') {
                    byte = byte.wrapping_mul(8).wrapping_add(bytes[index] - b'0');
                    index += 1;
                    digits += 1;
                }
                decoded.push(byte);
            }
            _ => return None,
        }
    }
    if let Some(nul) = decoded.iter().position(|byte| *byte == 0) {
        decoded.truncate(nul);
    }
    Some(decoded)
}

struct TerminalOptionValue<'a> {
    name: &'a str,
    index: Option<u32>,
    value: &'a str,
}

fn terminal_option_values<'a>(
    options: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<TerminalOptionValue<'a>> {
    let mut values = options
        .into_iter()
        .filter_map(|(key, value)| {
            ["terminal-features", "terminal-overrides"]
                .into_iter()
                .find_map(|name| {
                    if key == name {
                        return Some(TerminalOptionValue {
                            name,
                            index: None,
                            value,
                        });
                    }
                    let index = key
                        .strip_prefix(name)?
                        .strip_prefix('[')?
                        .strip_suffix(']')?
                        .parse::<u32>()
                        .ok()?;
                    Some(TerminalOptionValue {
                        name,
                        index: Some(index),
                        value,
                    })
                })
        })
        .collect::<Vec<_>>();

    for (name, defaults) in [
        ("terminal-features", DEFAULT_TERMINAL_FEATURES),
        ("terminal-overrides", DEFAULT_TERMINAL_OVERRIDES),
    ] {
        if values
            .iter()
            .any(|value| value.name == name && value.index.is_none())
        {
            continue;
        }
        for (index, default) in defaults.iter().enumerate() {
            let index = index as u32;
            if values
                .iter()
                .any(|value| value.name == name && value.index == Some(index))
            {
                continue;
            }
            values.push(TerminalOptionValue {
                name,
                index: Some(index),
                value: default,
            });
        }
    }
    values.sort_by_key(|value| (value.name, value.index));
    values
}

fn split_array_entries(value: &str) -> impl Iterator<Item = &str> {
    value.split(',').filter(|entry| !entry.is_empty())
}

/// Split tmux override fields. A doubled colon represents one literal colon.
fn split_override_fields(value: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == ':' {
            if chars.peek() == Some(&':') {
                chars.next();
                fields.last_mut().expect("one field").push(':');
            } else {
                fields.push(String::new());
            }
        } else {
            fields.last_mut().expect("one field").push(ch);
        }
    }
    fields
}

fn terminal_pattern_matches(pattern: &str, terminal: &str) -> bool {
    let (Ok(pattern), Ok(terminal)) = (CString::new(pattern), CString::new(terminal)) else {
        return false;
    };
    // SAFETY: both arguments are live NUL-terminated strings for the call.
    unsafe { libc::fnmatch(pattern.as_ptr(), terminal.as_ptr(), 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(name: &str, capabilities: &[&str]) -> TerminalIdentity {
        TerminalIdentity::new(
            name,
            capabilities
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            0,
            None,
        )
    }

    #[test]
    fn identified_capabilities_are_typed() {
        let term = ResolvedTerm::resolve(
            identity("screen", &["am=1", "colors=256", "clear=\x1b[H\x1b[2J"]),
            [],
        );
        assert!(term.auto_margin());
        assert_eq!(
            term.capability(Capability::colors),
            Some(&CapabilityValue::Number(256))
        );
        assert_eq!(
            term.capability(Capability::clear),
            Some(&CapabilityValue::String(b"\x1b[H\x1b[2J".to_vec()))
        );
    }

    #[test]
    fn complete_catalog_types_identify_values_and_ignores_unknown_entries() {
        let term = ResolvedTerm::resolve(
            identity(
                "dumb",
                &[
                    "Sxl=0",
                    "U8=1",
                    "kf63=last-key",
                    "bel=beep$<5/>done",
                    "colors=-1",
                    "not-a-tmux-capability=value",
                ],
            ),
            [],
        );
        assert_eq!(term.capability(Capability::Sxl), Some(&CapabilityValue::Flag(false)));
        assert_eq!(term.capability(Capability::U8), Some(&CapabilityValue::Number(1)));
        assert_eq!(
            term.capability(Capability::kf63),
            Some(&CapabilityValue::String(b"last-key".to_vec()))
        );
        assert_eq!(
            term.capability(Capability::bel),
            Some(&CapabilityValue::String(b"beepdone".to_vec()))
        );
        assert_eq!(term.capability(Capability::colors), None);
        assert_eq!(Capability::from_name("not-a-tmux-capability"), None);
    }

    #[test]
    fn overrides_match_term_and_apply_in_order() {
        let options = [("terminal-overrides", "xterm*:am@,screen*:am@,screen*:am")];
        let screen = ResolvedTerm::resolve(identity("screen-256color", &["am=1"]), options);
        let xterm = ResolvedTerm::resolve(identity("xterm-256color", &["am=1"]), options);
        assert!(screen.auto_margin(), "later matching entry restores am");
        assert!(!xterm.auto_margin(), "matching removal clears am");
    }

    #[test]
    fn screen_family_overline_inference_respects_explicit_removal() {
        let inferred = ResolvedTerm::resolve(identity("screen-256color", &[]), []);
        assert!(inferred.capability(Capability::Smol).is_some());

        let removed = ResolvedTerm::resolve(
            identity("screen-256color", &[]),
            [("terminal-overrides", "screen*:Smol@")],
        );
        assert!(removed.capability(Capability::Smol).is_none());
    }

    #[test]
    fn indexed_overrides_use_numeric_order() {
        let options = [
            ("terminal-overrides[10]", "screen*:am"),
            ("terminal-overrides[2]", "screen*:am@"),
        ];
        let term = ResolvedTerm::resolve(identity("screen", &["am=1"]), options);
        assert!(term.auto_margin());
    }

    #[test]
    fn configured_and_identified_features_are_client_scoped() {
        let mut identified = identity("xterm-256color", &[]);
        identified.feature_bits = 1 << 1; // bpaste
        identified.colorterm = Some("truecolor".into());
        let term = ResolvedTerm::resolve(
            identified,
            [("terminal-features", "screen*:title,xterm*:clipboard")],
        );
        assert!(term.has_feature("bpaste"));
        assert!(term.has_feature("RGB"));
        assert!(term.has_feature("clipboard"));
        assert!(!term.has_feature("title"));
    }

    #[test]
    fn tmux_default_feature_and_override_arrays_are_resolved() {
        let xterm = ResolvedTerm::resolve(identity("xterm-256color", &[]), []);
        for feature in ["clipboard", "ccolour", "cstyle", "focus", "title"] {
            assert!(
                xterm.has_feature(feature),
                "missing default feature {feature}"
            );
        }
        assert!(xterm.capability(Capability::Ms).is_some());
        assert!(xterm.capability(Capability::Ss).is_some());

        let linux = ResolvedTerm::resolve(identity("linux", &["AX=1"]), []);
        assert_eq!(linux.capability(Capability::AX), None);
    }

    #[test]
    fn indexed_option_entries_replace_the_same_default_index() {
        let term = ResolvedTerm::resolve(
            identity("xterm", &[]),
            [("terminal-features[0]", "other:clipboard")],
        );
        assert!(!term.has_feature("clipboard"));
        assert!(!term.has_feature("ccolour"));
    }

    #[test]
    fn feature_presets_install_capabilities_before_final_overrides() {
        let term = ResolvedTerm::resolve(
            identity("xterm", &[]),
            [
                ("terminal-features", "xterm:RGB:sync:usstyle"),
                (
                    "terminal-overrides",
                    r"xterm:setrgbf=custom:setrgbb@:Sync=\E[custom",
                ),
            ],
        );
        assert!(term.has_feature("RGB"));
        assert!(term.has_feature("sync"));
        assert_eq!(
            term.capability(Capability::setrgbf),
            Some(&CapabilityValue::String(b"custom".to_vec()))
        );
        assert_eq!(term.capability(Capability::setrgbb), None);
        assert_eq!(
            term.capability(Capability::Sync),
            Some(&CapabilityValue::String(b"\x1b[custom".to_vec()))
        );
        assert_eq!(
            term.capability(Capability::Smulx),
            Some(&CapabilityValue::String(b"\x1b[4:%p1%dm".to_vec()))
        );
    }

    #[test]
    fn vt100_and_rgb_capabilities_infer_tmux_features() {
        let term = ResolvedTerm::resolve(identity("xterm", &["clear=\x1b[H\x1b[2J", "Tc=1"]), []);
        for feature in ["bpaste", "focus", "title", "RGB"] {
            assert!(
                term.has_feature(feature),
                "missing inferred feature {feature}"
            );
        }
        assert_eq!(
            term.capability(Capability::Enbp),
            Some(&CapabilityValue::String(b"\x1b[?2004h".to_vec()))
        );
        assert!(term.capability(Capability::setrgbf).is_some());
        assert!(term.terminal_flags & TERM_VT100_LIKE != 0);
        assert!(term.terminal_flags & TERM_RGB_COLOURS != 0);
    }

    #[test]
    fn unknown_feature_stops_the_current_feature_list() {
        let term = ResolvedTerm::resolve(
            identity("screen", &[]),
            [("terminal-features", "screen:mouse:unknown:title")],
        );
        assert!(term.has_feature("mouse"));
        assert!(!term.has_feature("title"));
        assert!(term.capability(Capability::kmous).is_some());
        assert!(term.capability(Capability::tsl).is_none());
    }

    #[test]
    fn ignorefkeys_removes_the_catalogued_function_keys() {
        let mut identified = identity("rxvt", &["kf1=one", "kf12=twelve", "kf63=last"]);
        identified.feature_bits = 1 << 8;
        let term = ResolvedTerm::resolve(identified, []);
        assert!(term.has_feature("ignorefkeys"));
        assert_eq!(term.capability(Capability::kf1), None);
        assert_eq!(term.capability(Capability::kf12), None);
        assert_eq!(term.capability(Capability::kf63), None);
    }

    #[test]
    fn features_can_be_detected_from_existing_capabilities() {
        let term = ResolvedTerm::resolve(
            identity("dumb", &["Enbp=on", "Dsbp=off", "kmous=mouse"]),
            [],
        );
        assert!(term.has_feature("bpaste"));
        assert!(term.has_feature("mouse"));
        assert!(!term.has_feature("ignorefkeys"));
    }

    #[test]
    fn acs_uses_terminfo_pairs_or_tmux_ascii_fallback() {
        let identified = ResolvedTerm::resolve(identity("screen", &["acsc=q-x|"]), []);
        assert_eq!(identified.acs(b'q'), Some(b'-'));
        assert_eq!(identified.acs(b'x'), Some(b'|'));

        let fallback = ResolvedTerm::resolve(identity("dumb", &[]), []);
        assert_eq!(fallback.acs(b'j'), Some(b'+'));
        assert_eq!(fallback.acs(b'x'), Some(b'|'));
    }

    #[test]
    fn override_values_decode_vis_sequences_and_escaped_colons() {
        let term = ResolvedTerm::resolve(
            identity("screen", &[]),
            [("terminal-overrides", r"screen:bel=\E]1::ready\007")],
        );
        assert_eq!(
            term.capability(Capability::bel),
            Some(&CapabilityValue::String(b"\x1b]1:ready\x07".to_vec()))
        );
    }

    #[test]
    fn vis_decoding_matches_tmux_meta_hidden_and_error_rules() {
        assert_eq!(decode_vis(r"\M-A"), Some(vec![0xc1]));
        assert_eq!(
            decode_vis("left\\\nright\\$done"),
            Some(b"leftrightdone".to_vec())
        );
        assert_eq!(decode_vis(r"left\000right"), Some(b"left".to_vec()));
        assert_eq!(decode_vis(r"\e"), None);
        assert_eq!(decode_vis(r"\x1b"), None);
    }

    #[test]
    fn parameterized_strings_expand_numeric_and_string_arguments() {
        let term = ResolvedTerm::resolve(
            identity(
                "screen",
                &[
                    "colors=256",
                    "cup=\x1b[%i%p1%d;%p2%dH",
                    "Ms=\x1b]52;%p1%s;%p2%s\x07",
                ],
            ),
            [],
        );
        assert_eq!(number_capability(&term, Capability::colors), Some(256));
        assert_eq!(
            expand_capability(
                &term,
                Capability::cup,
                &[
                    CapabilityParameter::Number(0),
                    CapabilityParameter::Number(4),
                ],
            ),
            Some(b"\x1b[1;5H".to_vec())
        );
        assert_eq!(
            expand_capability(
                &term,
                Capability::Ms,
                &[
                    CapabilityParameter::String("c"),
                    CapabilityParameter::String("aGVsbG8="),
                ],
            ),
            Some(b"\x1b]52;c;aGVsbG8=\x07".to_vec())
        );
        assert_eq!(expand_capability(&term, Capability::colors, &[]), None);
    }

    #[test]
    fn feature_strings_expand_tmux_conditionals_and_arithmetic() {
        let term = ResolvedTerm::resolve(
            identity("modern", &[]),
            [("terminal-features", "modern:RGB:hyperlinks:sync:usstyle")],
        );
        for (colour, expected) in [
            (1, b"\x1b[31m".as_slice()),
            (8, b"\x1b[90m".as_slice()),
            (16, b"\x1b[38;5;16m".as_slice()),
        ] {
            assert_eq!(
                expand_capability(&term, Capability::setaf, &[CapabilityParameter::Number(colour)]),
                Some(expected.to_vec())
            );
        }
        assert_eq!(
            expand_capability(&term, Capability::Sync, &[CapabilityParameter::Number(1)]),
            Some(b"\x1b[?2026h".to_vec())
        );
        assert_eq!(
            expand_capability(&term, Capability::Sync, &[CapabilityParameter::Number(2)]),
            Some(b"\x1b[?2026l".to_vec())
        );
        assert_eq!(
            expand_capability(
                &term,
                Capability::Hls,
                &[
                    CapabilityParameter::String("link"),
                    CapabilityParameter::String("https://example.test"),
                ],
            ),
            Some(b"\x1b]8;id=link;https://example.test\x1b\\".to_vec())
        );
    }

    #[test]
    fn setulc_stray_conditional_matches_tmux() {
        let term = ResolvedTerm::resolve(
            identity("modern", &[]),
            [("terminal-features", "modern:RGB:hyperlinks:sync:usstyle")],
        );
        assert_eq!(
            expand_capability(
                &term,
                Capability::Setulc,
                &[CapabilityParameter::Number(0x11_22_33)]
            ),
            Some(b"\x1b[58:2:17:34:51m".to_vec())
        );
    }

    #[test]
    fn conditional_normalization_preserves_escaped_percent_sequences() {
        assert_eq!(
            strip_unmatched_conditional_ends(b"%%?literal%%;%;"),
            b"%%?literal%%;"
        );
        assert_eq!(
            strip_unmatched_conditional_ends(b"%?%p1%ttrue%;%;"),
            b"%?%p1%ttrue%;"
        );
    }

    #[test]
    fn clear_and_cup_are_required_for_an_attached_terminal() {
        let missing_both = ResolvedTerm::resolve(identity("dumb", &[]), []);
        assert_eq!(
            missing_both.validation_error(),
            Some("terminal does not support clear")
        );

        let missing_cup = ResolvedTerm::resolve(identity("dumb", &["clear=clear"]), []);
        assert_eq!(
            missing_cup.validation_error(),
            Some("terminal does not support cup")
        );

        let complete = ResolvedTerm::resolve(identity("dumb", &["clear=clear", "cup=move"]), []);
        assert_eq!(complete.validation_error(), None);
    }

    #[test]
    fn only_the_physical_bottom_row_loses_a_cell_without_am() {
        let term = ResolvedTerm::resolve(identity("screen", &["am=0"]), []);
        assert_eq!(writable_width(&term, 1, 20, 2), 20);
        assert_eq!(writable_width(&term, 2, 20, 2), 19);
    }
}
