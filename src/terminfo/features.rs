use super::term::TerminalCapabilities;
use crate::fmt_args;
use crate::log::log_debug;
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};

pub use crate::consts::{TERM_256COLOURS, TERM_DECFRA, TERM_DECSLRM, TERM_RGBCOLOURS, TERM_SIXEL};

/// Parses and names terminal feature bitsets.
pub trait TerminalFeatureSet {
    /// Adds the recognized names from `features` to `current`.
    fn add(&self, current: c_int, features: &CStr, separators: &CStr) -> c_int;

    /// Returns the names present in `features`, in bit order.
    fn names(&self, features: c_int) -> std::ffi::CString;

    /// Adds the defaults associated with a terminal name and version.
    fn defaults(&self, current: c_int, name: &CStr, version: u_int) -> c_int;
}

/// Terminal feature-set operations implemented by hmux.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustTerminalFeatureSet;

impl TerminalFeatureSet for RustTerminalFeatureSet {
    fn add(&self, mut current: c_int, features: &CStr, separators: &CStr) -> c_int {
        tty_add_features_impl(&mut current, features, separators);
        current
    }

    fn names(&self, features: c_int) -> std::ffi::CString {
        tty_get_features_impl(features)
    }

    fn defaults(&self, mut current: c_int, name: &CStr, version: u_int) -> c_int {
        tty_default_features_impl(&mut current, name, version);
        current
    }
}

/// One terminal feature: the name a user writes, the terminfo capabilities it
/// stands for and the terminal flags it carries. A feature's bit is its index
/// in [`tty_features`], so the order below is the wire order of the feature
/// flags a client sends.
pub struct tty_feature {
    pub name: &'static CStr,
    pub capabilities: &'static [&'static CStr],
    pub flags: c_int,
}

/// The sixty-four function keys `ignorefkeys` takes away. A capability ending
/// in `@` removes it rather than setting it.
static tty_feature_ignorefkeys_capabilities: [&CStr; 64] = [
    c"kf0@", c"kf1@", c"kf2@", c"kf3@", c"kf4@", c"kf5@", c"kf6@", c"kf7@", c"kf8@", c"kf9@",
    c"kf10@", c"kf11@", c"kf12@", c"kf13@", c"kf14@", c"kf15@", c"kf16@", c"kf17@", c"kf18@",
    c"kf19@", c"kf20@", c"kf21@", c"kf22@", c"kf23@", c"kf24@", c"kf25@", c"kf26@", c"kf27@",
    c"kf28@", c"kf29@", c"kf30@", c"kf31@", c"kf32@", c"kf33@", c"kf34@", c"kf35@", c"kf36@",
    c"kf37@", c"kf38@", c"kf39@", c"kf40@", c"kf41@", c"kf42@", c"kf43@", c"kf44@", c"kf45@",
    c"kf46@", c"kf47@", c"kf48@", c"kf49@", c"kf50@", c"kf51@", c"kf52@", c"kf53@", c"kf54@",
    c"kf55@", c"kf56@", c"kf57@", c"kf58@", c"kf59@", c"kf60@", c"kf61@", c"kf62@", c"kf63@",
];

/// Every feature a terminal can be given, in bit order.
static tty_features: [tty_feature; 21] = [
    tty_feature {
        name: c"256",
        capabilities: &[
            c"AX",
            c"setab=\\E[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m",
            c"setaf=\\E[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m",
        ],
        flags: TERM_256COLOURS,
    },
    tty_feature {
        name: c"bpaste",
        capabilities: &[c"Enbp=\\E[?2004h", c"Dsbp=\\E[?2004l"],
        flags: 0,
    },
    tty_feature {
        name: c"ccolour",
        capabilities: &[c"Cs=\\E]12;%p1%s\\a", c"Cr=\\E]112\\a"],
        flags: 0,
    },
    tty_feature {
        name: c"clipboard",
        capabilities: &[c"Ms=\\E]52;%p1%s;%p2%s\\a"],
        flags: 0,
    },
    tty_feature {
        name: c"hyperlinks",
        capabilities: &[c"Hls=\\E]8;%?%p1%l%tid=%p1%s%;;%p2%s\\E\\\\"],
        flags: 0,
    },
    tty_feature {
        name: c"cstyle",
        capabilities: &[c"Ss=\\E[%p1%d q", c"Se=\\E[2 q"],
        flags: 0,
    },
    tty_feature {
        name: c"extkeys",
        capabilities: &[c"Eneks=\\E[>4;2m", c"Dseks=\\E[>4m"],
        flags: 0,
    },
    tty_feature {
        name: c"focus",
        capabilities: &[c"Enfcs=\\E[?1004h", c"Dsfcs=\\E[?1004l"],
        flags: 0,
    },
    tty_feature {
        name: c"ignorefkeys",
        capabilities: &tty_feature_ignorefkeys_capabilities,
        flags: 0,
    },
    tty_feature {
        name: c"margins",
        capabilities: &[
            c"Enmg=\\E[?69h",
            c"Dsmg=\\E[?69l",
            c"Clmg=\\E[s",
            c"Cmg=\\E[%i%p1%d;%p2%ds",
        ],
        flags: TERM_DECSLRM,
    },
    tty_feature {
        name: c"mouse",
        capabilities: &[c"kmous=\\E[M"],
        flags: 0,
    },
    tty_feature {
        name: c"osc7",
        capabilities: &[c"Swd=\\E]7;", c"fsl=\\a"],
        flags: 0,
    },
    tty_feature {
        name: c"overline",
        capabilities: &[c"Smol=\\E[53m"],
        flags: 0,
    },
    tty_feature {
        name: c"progressbar",
        capabilities: &[c"Spb=\\E]9;4;%p1%d;%p2%d\\E\\\\"],
        flags: 0,
    },
    tty_feature {
        name: c"rectfill",
        capabilities: &[c"Rect"],
        flags: TERM_DECFRA,
    },
    tty_feature {
        name: c"RGB",
        capabilities: &[
            c"AX",
            c"setrgbf=\\E[38;2;%p1%d;%p2%d;%p3%dm",
            c"setrgbb=\\E[48;2;%p1%d;%p2%d;%p3%dm",
            c"setab=\\E[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m",
            c"setaf=\\E[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m",
        ],
        flags: TERM_256COLOURS | TERM_RGBCOLOURS,
    },
    tty_feature {
        name: c"sixel",
        capabilities: &[c"Sxl"],
        flags: TERM_SIXEL,
    },
    tty_feature {
        name: c"strikethrough",
        capabilities: &[c"smxx=\\E[9m"],
        flags: 0,
    },
    tty_feature {
        name: c"sync",
        capabilities: &[c"Sync=\\E[?2026%?%p1%{1}%-%tl%eh%;"],
        flags: 0,
    },
    tty_feature {
        name: c"title",
        capabilities: &[c"tsl=\\E]0;", c"fsl=\\a"],
        flags: 0,
    },
    tty_feature {
        name: c"usstyle",
        capabilities: &[
            c"Smulx=\\E[4::%p1%dm",
            c"Setulc=\\E[58::2::%p1%{65536}%/%d::%p1%{256}%/%{255}%&%d::%p1%{255}%&%d%;m",
            c"Setulc1=\\E[58::5::%p1%dm",
            c"ol=\\E[59m",
        ],
        flags: 0,
    },
];

/// Adds the features `s` names to the bits behind `feat`. A name runs up to
/// any byte of `separators` and is matched without regard to case.
///
/// A name the table does not carry stops the whole list, rather than being
/// skipped: `256,bogus,title` adds `256` and nothing else. That is what the C
/// does, and a client sending an unknown feature name loses the rest of its
/// list the same way.
fn tty_add_features_impl(feat: &mut c_int, s: &CStr, separators: &CStr) {
    {
        log_debug(c"adding terminal features %s", fmt_args![s.as_ptr()]);
        let separators = separators.to_bytes();
        for next in s.to_bytes().split(|byte| separators.contains(byte)) {
            let found = tty_features
                .iter()
                .position(|tf| tf.name.to_bytes().eq_ignore_ascii_case(next));
            let Some(i) = found else {
                log_debug(
                    c"unknown terminal feature: %.*s",
                    fmt_args![next.len() as c_int, next.as_ptr()],
                );
                break;
            };
            if *feat & (1 << i) == 0 {
                log_debug(
                    c"adding terminal feature: %s",
                    fmt_args![tty_features[i].name.as_ptr()],
                );
                *feat |= 1 << i;
            }
        }
    }
}

/// Names the features `feat` carries, comma-separated in bit order, as the
/// caller's own string.
fn tty_get_features_impl(feat: c_int) -> std::ffi::CString {
    let mut names = Vec::<u8>::new();
    for (i, tf) in tty_features.iter().enumerate() {
        if feat & (1 << i) != 0 {
            if !names.is_empty() {
                names.push(b',');
            }
            names.extend_from_slice(tf.name.to_bytes());
        }
    }
    std::ffi::CString::new(names).expect("a feature name has no interior NUL")
}

/// Gives `term` the capabilities and terminal flags of every feature in `feat`
/// it does not have yet, and answers whether that added anything to the
/// terminal's own feature set.
///
/// Every feature in the table names at least one capability, so the C's check
/// for a feature with none is gone.
pub(crate) unsafe fn tty_apply_features(term: &mut tty_term, feat: c_int) -> c_int {
    {
        if feat == 0 {
            return 0;
        }
        log_debug(
            c"applying terminal features: %s",
            fmt_args![RustTerminalFeatureSet.names(feat).as_c_str()],
        );
        for (i, tf) in tty_features.iter().enumerate() {
            if term.features & (1 << i) != 0 || feat & (1 << i) == 0 {
                continue;
            }
            log_debug(
                c"applying terminal feature: %s",
                fmt_args![tf.name.as_ptr()],
            );
            for capability in tf.capabilities {
                log_debug(c"adding capability: %s", fmt_args![capability.as_ptr()]);
                term.apply_overrides(capability);
            }
            term.flags |= tf.flags;
        }
        if term.features | feat == term.features {
            return 0;
        }
        term.features |= feat;
        1
    }
}

/// The features a terminal calling itself `name` is known to have, whatever
/// its terminfo entry says.
///
/// `version` is what the terminal reported for itself. The C turned an entry
/// down when the terminal was older than the entry asked for, but every entry
/// below asks for version zero, so no terminal was ever turned down and the
/// check is gone.
fn tty_default_features_impl(feat: &mut c_int, name: &CStr, _version: u_int) {
    static table: [(&CStr, &CStr); 7] = [
        (
            c"mintty",
            c"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,margins,overline,usstyle",
        ),
        (
            c"tmux",
            c"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,overline,usstyle,hyperlinks,progressbar",
        ),
        (
            c"rxvt-unicode",
            c"256,bpaste,ccolour,cstyle,mouse,title,ignorefkeys",
        ),
        (
            c"iTerm2",
            c"256,RGB,bpaste,clipboard,mouse,strikethrough,title,cstyle,extkeys,margins,usstyle,sync,osc7,hyperlinks,progressbar",
        ),
        (
            c"foot",
            c"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,usstyle,sync,osc7,hyperlinks",
        ),
        (
            c"WezTerm",
            c"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,usstyle",
        ),
        (
            c"XTerm",
            c"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus",
        ),
    ];
    for (entry, features) in table {
        if entry == name {
            tty_add_features_impl(feat, features, c",");
        }
    }
}

pub fn tty_add_features(feat: &mut c_int, features: &CStr, separators: &CStr) {
    *feat = RustTerminalFeatureSet.add(*feat, features, separators);
}

pub fn tty_get_features(features: c_int) -> std::ffi::CString {
    RustTerminalFeatureSet.names(features)
}

pub fn tty_default_features(feat: &mut c_int, name: &CStr, version: u_int) {
    *feat = RustTerminalFeatureSet.defaults(*feat, name, version);
}
#[cfg(test)]
#[path = "../tests/test_tty_features.rs"]
mod tests;
