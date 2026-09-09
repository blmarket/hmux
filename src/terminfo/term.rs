use super::features::tty_apply_features;
use super::features::{RustTerminalFeatureSet, TerminalFeatureSet};
use crate::compat::strnvis;
use crate::compat::strtonum;
use crate::environ::EnvironmentStore;
use crate::ffi::{
    cur_term, del_curterm, fnmatch, setupterm, tigetflag, tigetnum, tigetstr, tiparm_s,
};
use crate::fmt_args;
use crate::fmt_engine::format_alloc;
use crate::log::{fatalx, log_debug};
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
use crate::tmux::global_options;
use crate::tty::tty_client;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;

/// An immutable observation of one terminal capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalCapabilityRef<'a> {
    /// The terminal does not provide the capability.
    Missing,
    /// A string capability.
    String(&'a CStr),
    /// A numeric capability.
    Number(core::ffi::c_int),
    /// A boolean capability. A false value is still present.
    Flag(bool),
}

/// A terminal capability description independent of its representation.
pub trait TerminalCapabilities: Sized {
    /// Makes an independent description with every capability missing.
    fn new(name: &CStr) -> Self;

    /// Returns the terminal type name.
    fn name(&self) -> &CStr;

    /// Returns the number of capability slots in tmux's table.
    fn capability_count(&self) -> u_int;

    /// Returns the terminfo name of one capability slot.
    fn capability_name(&self, code: tty_code_code) -> &CStr;

    /// Finds a capability slot by its terminfo name.
    fn find_capability(&self, name: &CStr) -> Option<tty_code_code> {
        (0..self.capability_count())
            .map(|code| code as tty_code_code)
            .find(|&code| self.capability_name(code) == name)
    }

    /// Observes one capability slot.
    fn capability(&self, code: tty_code_code) -> TerminalCapabilityRef<'_>;

    /// Returns whether the terminal provides `code`.
    fn has(&self, code: tty_code_code) -> bool {
        !matches!(self.capability(code), TerminalCapabilityRef::Missing)
    }

    /// Returns a string capability, or an empty string when it is missing.
    fn string(&self, code: tty_code_code) -> &CStr {
        match self.capability(code) {
            TerminalCapabilityRef::Missing => c"",
            TerminalCapabilityRef::String(value) => value,
            _ => panic!("terminal capability {code} is not a string"),
        }
    }

    /// Returns a numeric capability, or zero when it is missing.
    fn number(&self, code: tty_code_code) -> core::ffi::c_int {
        match self.capability(code) {
            TerminalCapabilityRef::Missing => 0,
            TerminalCapabilityRef::Number(value) => value,
            _ => panic!("terminal capability {code} is not a number"),
        }
    }

    /// Returns a flag capability as zero or one, or zero when it is missing.
    fn flag(&self, code: tty_code_code) -> core::ffi::c_int {
        match self.capability(code) {
            TerminalCapabilityRef::Missing => 0,
            TerminalCapabilityRef::Flag(value) => core::ffi::c_int::from(value),
            _ => panic!("terminal capability {code} is not a flag"),
        }
    }

    /// Applies tmux's colon-separated terminal override syntax.
    fn apply_overrides(&mut self, capabilities: &CStr);

    /// Applies feature bits not previously applied.
    fn apply_features(&mut self, features: core::ffi::c_int) -> bool;

    /// Recomputes capability-derived flags and the ACS translation table.
    fn refresh_derived(&mut self);

    /// Returns the terminal's derived and feature flags.
    fn flags(&self) -> core::ffi::c_int;

    /// Returns the one-byte ACS translation for `ch`, if one exists.
    fn acs(&self, ch: u8) -> Option<&CStr>;

    /// Expands a string capability with one numeric argument.
    fn expand_i(&self, code: tty_code_code, a: core::ffi::c_int) -> CString;

    /// Expands a string capability with two numeric arguments.
    fn expand_ii(&self, code: tty_code_code, a: core::ffi::c_int, b: core::ffi::c_int) -> CString;

    /// Expands a string capability with three numeric arguments.
    fn expand_iii(
        &self,
        code: tty_code_code,
        a: core::ffi::c_int,
        b: core::ffi::c_int,
        c: core::ffi::c_int,
    ) -> CString;

    /// Expands a string capability with one string argument.
    fn expand_s(&self, code: tty_code_code, a: &CStr) -> CString;

    /// Expands a string capability with two string arguments.
    fn expand_ss(&self, code: tty_code_code, a: &CStr, b: &CStr) -> CString;

    /// Describes one capability in the form used by `show-messages -T`.
    fn describe(&self, code: tty_code_code) -> CString;
}

/// The terminal capability implementation used by hmux.
pub type RustTerminalCapabilities = tty_term;
/// What a terminal says one capability is: the entry is either missing or
/// carries the value its table entry calls for.
pub(crate) enum TtyCode {
    None,
    String(CString),
    Number(core::ffi::c_int),
    Flag(core::ffi::c_int),
}
pub const TTYCODE_FLAG: tty_code_type = 3;
pub const TTYCODE_NUMBER: tty_code_type = 2;
pub const TTYCODE_STRING: tty_code_type = 1;
pub const TTYCODE_NONE: tty_code_type = 0;
pub use crate::consts::{
    INT_MAX, TERM_DECFRA, TERM_DECSLRM, TERM_NOAM, TERM_RGBCOLOURS, TERM_SIXEL, TERM_VT100LIKE,
    TTYC_ACSC, TTYC_AM, TTYC_CLEAR, TTYC_CLMG, TTYC_CMG, TTYC_CUP, TTYC_RGB, TTYC_SETRGBB,
    TTYC_SETRGBF, TTYC_XT, VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB,
};
pub const TTYC_TC: tty_code_code = 228;

pub const TTYC_RECT: tty_code_code = 197;

pub const OK: core::ffi::c_int = 0 as core::ffi::c_int;

/// A terminal description retained independently of its tty, with checked borrows.
pub(crate) type TerminalRef = std::rc::Rc<std::cell::RefCell<tty_term>>;

/// One terminal description the server holds, and the client whose tty owns
/// it.
struct tty_term_entry {
    identity: std::rc::Rc<()>,
    term: std::rc::Weak<std::cell::RefCell<tty_term>>,
    client: Option<ClientWeak>,
}

/// Every terminal description the server holds, newest first. Each one is
/// owned by the tty of the client it was made for; this is the observer list
/// `show-messages -T` walks.
#[derive(Default)]
struct TerminalRegistry {
    entries: std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<tty_term_entry>>>,
}

pub(crate) struct TerminalRegistration {
    entries: std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<tty_term_entry>>>,
    identity: std::rc::Rc<()>,
}

impl TerminalRegistry {
    fn register(&self, term: &TerminalRef, client: Option<ClientWeak>) -> TerminalRegistration {
        let identity = std::rc::Rc::new(());
        self.entries.borrow_mut().push_front(tty_term_entry {
            identity: identity.clone(),
            term: std::rc::Rc::downgrade(term),
            client,
        });
        TerminalRegistration {
            entries: self.entries.clone(),
            identity,
        }
    }
}

impl Drop for TerminalRegistration {
    fn drop(&mut self) {
        let removed = {
            let mut entries = self.entries.borrow_mut();
            let at = entries
                .iter()
                .position(|entry| std::rc::Rc::ptr_eq(&entry.identity, &self.identity));
            at.and_then(|at| entries.remove(at))
        };
        drop(removed);
    }
}

impl Drop for tty_term {
    fn drop(&mut self) {
        drop(self.registration.take());
    }
}

thread_local! {
    static TTY_TERMS: TerminalRegistry = TerminalRegistry::default();
}

pub(crate) struct TerminalDescriptionSnapshot {
    pub(crate) name: CString,
    pub(crate) client_name: Option<CString>,
    pub(crate) flags: core::ffi::c_int,
    pub(crate) capabilities: Vec<CString>,
}

pub(crate) fn tty_term_snapshots(target: Option<&tty_term>) -> Vec<TerminalDescriptionSnapshot> {
    TTY_TERMS.with(|registry| {
        registry
            .entries
            .borrow()
            .iter()
            .filter(|listed| {
                target.is_none_or(|target| {
                    target.registration.as_ref().is_some_and(|registration| {
                        std::rc::Rc::ptr_eq(&listed.identity, &registration.identity)
                    })
                })
            })
            .filter_map(|listed| {
                let owner = listed.term.upgrade()?;
                let term = owner.borrow();
                Some(TerminalDescriptionSnapshot {
                    name: term.name().to_owned(),
                    client_name: listed
                        .client
                        .as_ref()
                        .and_then(ClientWeak::upgrade)
                        .and_then(|client| unsafe { client.name().map(std::ffi::CStr::to_owned) }),
                    flags: term.flags(),
                    capabilities: (0..term.capability_count())
                        .map(|code| term.describe(code as tty_code_code))
                        .collect(),
                })
            })
            .collect()
    })
}
static tty_term_codes: [tty_term_code_entry; 233] = [
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"acsc",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"am",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"AX",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"bce",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"bel",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Bidi",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"blink",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"bold",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"civis",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"clear",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Clmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Cmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cnorm",
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: c"colors",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Cr",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Cs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"csr",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cub",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cub1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cud",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cud1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuf",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuf1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cup",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuu",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cuu1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"cvvis",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dch",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dch1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dim",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"dl1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dsbp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dseks",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dsfcs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Dsmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"E3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ech",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ed",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"el",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"el1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"enacs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Enbp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Eneks",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Enfcs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Enmg",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"fsl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Hls",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"home",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"hpa",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ich",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ich1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"il",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"il1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"indn",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"invis",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcbt",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcub1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcud1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcuf1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kcuu1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDC7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kdch1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kDN7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kend",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kEND7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf10",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf11",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf12",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf13",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf14",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf15",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf16",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf17",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf18",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf19",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf2",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf20",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf21",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf22",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf23",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf24",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf25",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf26",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf27",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf28",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf29",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf30",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf31",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf32",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf33",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf34",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf35",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf36",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf37",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf38",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf39",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf40",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf41",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf42",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf43",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf44",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf45",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf46",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf47",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf48",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf49",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf50",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf51",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf52",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf53",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf54",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf55",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf56",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf57",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf58",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf59",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf60",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf61",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf62",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf63",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf8",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kf9",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kHOM7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"khome",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kIC7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kich1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kind",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kLFT7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kmous",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"knp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kNXT7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kpp",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kPRV7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kri",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kRIT7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP3",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP4",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP5",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP6",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"kUP7",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Ms",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Nobr",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ol",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"op",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Rect",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rev",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"RGB",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"ri",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rin",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rmacs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rmcup",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"rmkx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Se",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setab",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setaf",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setal",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setrgbb",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"setrgbf",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Setulc",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Setulc1",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"sgr0",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"sitm",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smacs",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smcup",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smkx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Smol",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smso",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smul",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Smulx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"smxx",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Spb",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"Sxl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Ss",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Swd",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"Sync",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"Tc",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"tsl",
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: c"U8",
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: c"vpa",
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: c"XT",
    },
];
fn tty_term_ncodes() -> u_int {
    size_of::<[tty_term_code_entry; 233]>().wrapping_div(size_of::<tty_term_code_entry>()) as u_int
}
/// The slot the terminal keeps for capability `code`.
fn tty_term_code(term: &tty_term, code: tty_code_code) -> &TtyCode {
    &term.codes[code as usize]
}

/// The same slot, to write a capability the terminal has been told about.
fn tty_term_code_mut(term: &mut tty_term, code: tty_code_code) -> &mut TtyCode {
    &mut term.codes[code as usize]
}
fn tty_term_strip(s: &CStr) -> CString {
    let bytes = s.to_bytes();
    if !bytes.contains(&b'$') {
        return s.to_owned();
    }
    let mut stripped = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'$' && bytes.get(at + 1) == Some(&b'<') {
            while at < bytes.len() && bytes[at] != b'>' {
                at += 1;
            }
            if bytes.get(at) == Some(&b'>') {
                at += 1;
            }
            if at == bytes.len() {
                break;
            }
        }
        stripped.push(bytes[at]);
        if stripped.len() == 8191 {
            break;
        }
        at += 1;
    }
    unsafe { CString::from_vec_unchecked(stripped) }
}
/// The next capability in a colon-separated override list, with `::` read as
/// one colon, as the caller's own string. Answers nothing at the end of the
/// list, or for a capability longer than the list format allows.
fn tty_term_override_next(s: &CStr, offset: &mut size_t) -> Option<CString> {
    const LONGEST: usize = 8191;
    let bytes = s.to_bytes();
    let mut value = Vec::<u8>::new();
    let mut at: size_t = *offset;
    if at >= bytes.len() {
        return None;
    }
    while at < bytes.len() {
        if bytes[at] == b':' {
            if bytes.get(at + 1) != Some(&b':') {
                break;
            }
            value.push(b':');
            at += 2;
        } else {
            value.push(bytes[at]);
            at += 1;
        }
        if value.len() == LONGEST {
            return None;
        }
    }
    *offset = if at < bytes.len() { at + 1 } else { at };
    Some(CString::new(value).expect("a capability has no interior NUL"))
}
fn tty_term_apply(term: &mut tty_term, capabilities: &CStr, quiet: core::ffi::c_int) {
    unsafe {
        let mut offset: size_t = 0 as size_t;
        while let Some(next) = tty_term_override_next(capabilities, &mut offset) {
            if next.to_bytes().is_empty() {
                continue;
            }
            let mut next = next.into_bytes_with_nul();
            let (capability, value) = if let Some(at) = next.iter().position(|&byte| byte == b'=') {
                next[at] = 0;
                let (capability, encoded) = next.split_at(at + 1);
                let capability = CStr::from_bytes_with_nul(capability)
                    .expect("the capability name retains its terminator");
                let encoded = CStr::from_bytes_with_nul(encoded)
                    .expect("the override value retains its terminator");
                (
                    capability,
                    Some(
                        RustUtf8VisModel
                            .decode_cstr(encoded)
                            .unwrap_or_else(|| encoded.to_owned()),
                    ),
                )
            } else {
                let remove = next[next.len() - 2] == b'@';
                if remove {
                    next.pop();
                    *next.last_mut().expect("the removal marker is present") = 0;
                }
                (
                    CStr::from_bytes_with_nul(&next)
                        .expect("the capability name retains its terminator"),
                    if remove {
                        None
                    } else {
                        Some(CString::default())
                    },
                )
            };
            if quiet == 0 {
                match value.as_deref() {
                    None => log_debug(
                        c"%s override: %s@",
                        fmt_args![term.name.as_deref(), capability],
                    ),
                    Some(value) if value.to_bytes().is_empty() => log_debug(
                        c"%s override: %s",
                        fmt_args![term.name.as_deref(), capability],
                    ),
                    Some(value) => log_debug(
                        c"%s override: %s=%s",
                        fmt_args![term.name.as_deref(), capability, value],
                    ),
                }
            }
            for (i, ent) in tty_term_codes.iter().enumerate() {
                if capability == ent.name {
                    let code = tty_term_code_mut(term, i as tty_code_code);
                    if let Some(value) = &value {
                        match ent.type_0 {
                            TTYCODE_STRING => {
                                *code = TtyCode::String(value.clone());
                            }
                            TTYCODE_NUMBER => {
                                if let Ok(n) = strtonum(
                                    value,
                                    0 as core::ffi::c_longlong,
                                    INT_MAX as core::ffi::c_longlong,
                                ) {
                                    *code = TtyCode::Number(n as core::ffi::c_int);
                                }
                            }
                            TTYCODE_FLAG => {
                                *code = TtyCode::Flag(1 as core::ffi::c_int);
                            }
                            _ => {}
                        }
                    } else {
                        *code = TtyCode::None;
                    }
                }
            }
        }
    }
}
pub(crate) unsafe fn tty_term_apply_overrides(term: &mut tty_term) {
    unsafe {
        global_options
            .as_ref()
            .expect("global options are initialized")
            .with_entry(c"terminal-overrides", true, |entry| {
                let entry = entry.expect("global array option is initialized");
                for index in RustOptionsEngine.array_indices(entry) {
                    let text = RustOptionsEngine
                        .value_string(RustOptionsEngine.array_get(entry, index).unwrap());
                    let mut offset = 0;
                    let first = tty_term_override_next(text, &mut offset);
                    if first
                        .as_ref()
                        .is_some_and(|first| fnmatch(first.as_ptr(), term.name().as_ptr(), 0) == 0)
                    {
                        let remaining =
                            CStr::from_bytes_with_nul(&text.to_bytes_with_nul()[offset..])
                                .expect("override suffix retains its terminator");
                        tty_term_apply(term, remaining, 0);
                    }
                }
            });
        tty_term_refresh_derived(term);
    }
}

fn tty_term_refresh_derived(term: &mut tty_term) {
    unsafe {
        log_debug(
            c"SIXEL flag is %d",
            fmt_args![(term.flags & TERM_SIXEL != 0) as core::ffi::c_int],
        );
        if tty_term_has(term, TTYC_SETRGBF) != 0 && tty_term_has(term, TTYC_SETRGBB) != 0 {
            term.flags |= TERM_RGBCOLOURS;
        } else {
            term.flags &= !TERM_RGBCOLOURS;
        }
        log_debug(
            c"RGBCOLOURS flag is %d",
            fmt_args![(term.flags & TERM_RGBCOLOURS != 0) as core::ffi::c_int],
        );
        if tty_term_has(term, TTYC_CMG) != 0 && tty_term_has(term, TTYC_CLMG) != 0 {
            term.flags |= TERM_DECSLRM;
        } else {
            term.flags &= !TERM_DECSLRM;
        }
        log_debug(
            c"DECSLRM flag is %d",
            fmt_args![(term.flags & TERM_DECSLRM != 0) as core::ffi::c_int],
        );
        if tty_term_has(term, TTYC_RECT) != 0 {
            term.flags |= TERM_DECFRA;
        } else {
            term.flags &= !TERM_DECFRA;
        }
        log_debug(
            c"DECFRA flag is %d",
            fmt_args![(term.flags & TERM_DECFRA != 0) as core::ffi::c_int],
        );
        if tty_term_flag(term, TTYC_AM) == 0 {
            term.flags |= TERM_NOAM;
        } else {
            term.flags &= !TERM_NOAM;
        }
        log_debug(
            c"NOAM flag is %d",
            fmt_args![(term.flags & TERM_NOAM != 0) as core::ffi::c_int],
        );
        let mut acs = [[0; 2]; 256];
        let mapping = if tty_term_has(term, TTYC_ACSC) != 0 {
            term.string(TTYC_ACSC)
        } else {
            c"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~."
        };
        for pair in mapping.to_bytes().as_chunks::<2>().0 {
            acs[pair[0] as usize][0] = pair[1];
        }
        term.acs = acs;
    }
}
pub(crate) fn tty_term_create(
    tty: &mut tty,
    name: &CStr,
    caps: &[CString],
    feat: &mut core::ffi::c_int,
) -> Result<TerminalRef, CString> {
    unsafe {
        log_debug(c"adding term %s", fmt_args![name]);
        let mut term = tty_term {
            name: Some(name.to_owned()),
            features: 0,
            acs: [[0; 2]; 256],
            codes: (0..tty_term_ncodes()).map(|_| TtyCode::None).collect(),
            flags: 0,
            registration: None,
        };
        for cap in caps {
            let Some(at) = cap.to_bytes().iter().position(|&byte| byte == b'=') else {
                continue;
            };
            let capability = &cap.to_bytes()[..at];
            let value = CStr::from_bytes_with_nul(&cap.to_bytes_with_nul()[at + 1..])
                .expect("a capability value retains its terminator");
            for (j, ent) in tty_term_codes.iter().enumerate() {
                if ent.name.to_bytes() == capability {
                    let code = tty_term_code_mut(&mut term, j as tty_code_code);
                    *code = TtyCode::None;
                    match ent.type_0 {
                        TTYCODE_STRING => {
                            *code = TtyCode::String(tty_term_strip(value));
                        }
                        TTYCODE_NUMBER => {
                            match strtonum(value, 0, INT_MAX as core::ffi::c_longlong) {
                                Ok(n) => *code = TtyCode::Number(n as core::ffi::c_int),
                                Err(errstr) => log_debug(c"%s: %s", fmt_args![ent.name, errstr]),
                            }
                        }
                        TTYCODE_FLAG => {
                            *code = TtyCode::Flag(
                                value.to_bytes().starts_with(b"1") as core::ffi::c_int
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        global_options
            .as_ref()
            .expect("global options are initialized")
            .with_entry(c"terminal-features", true, |entry| {
                let entry = entry.expect("global array option is initialized");
                for index in RustOptionsEngine.array_indices(entry) {
                    let text = RustOptionsEngine
                        .value_string(RustOptionsEngine.array_get(entry, index).unwrap());
                    let mut offset = 0;
                    let first = tty_term_override_next(text, &mut offset);
                    if first
                        .as_ref()
                        .is_some_and(|first| fnmatch(first.as_ptr(), term.name().as_ptr(), 0) == 0)
                    {
                        let remaining =
                            CStr::from_bytes_with_nul(&text.to_bytes_with_nul()[offset..])
                                .expect("override suffix retains its terminator");
                        *feat = RustTerminalFeatureSet.add(*feat, remaining, c":");
                    }
                }
            });
        del_curterm(cur_term);
        let owner = tty_client(tty).expect("the tty has a client");
        let colorterm = owner
            .environ_ref()
            .find(c"COLORTERM")
            .and_then(|entry| entry.value);
        if let Some(colorterm) = colorterm {
            log_debug(c"%s COLORTERM=%s", fmt_args![owner.name(), colorterm]);
            if colorterm.to_bytes().eq_ignore_ascii_case(b"truecolor")
                || colorterm.to_bytes().eq_ignore_ascii_case(b"24bit")
            {
                *feat = RustTerminalFeatureSet.add(*feat, c"RGB", c",");
            } else if bytes_have(colorterm.to_bytes(), b"256") {
                *feat = RustTerminalFeatureSet.add(*feat, c"256", c",");
            }
        }
        tty_term_apply_overrides(&mut term);
        let cause = if tty_term_has(&term, TTYC_CLEAR) == 0 {
            xasprintf(c"terminal does not support clear", fmt_args![])
        } else if tty_term_has(&term, TTYC_CUP) == 0 {
            xasprintf(c"terminal does not support cup", fmt_args![])
        } else {
            let clear = term.string(TTYC_CLEAR);
            if tty_term_flag(&term, TTYC_XT) != 0 || clear.to_bytes().starts_with(b"\x1B[") {
                term.flags |= TERM_VT100LIKE;
                *feat = RustTerminalFeatureSet.add(*feat, c"bpaste,focus,title", c",");
            }
            if (tty_term_flag(&term, TTYC_TC) != 0 || tty_term_has(&term, TTYC_RGB) != 0)
                && (tty_term_has(&term, TTYC_SETRGBF) == 0
                    || tty_term_has(&term, TTYC_SETRGBB) == 0)
            {
                *feat = RustTerminalFeatureSet.add(*feat, c"RGB", c",");
            }
            if tty_apply_features(&mut term, *feat) != 0 {
                tty_term_apply_overrides(&mut term);
            }
            for i in 0..tty_term_ncodes() {
                log_debug(
                    c"%s%s",
                    fmt_args![
                        name,
                        tty_term_describe(&term, i as tty_code_code).as_c_str()
                    ],
                );
            }
            let term = std::rc::Rc::new(std::cell::RefCell::new(term));
            let registration = TTY_TERMS.with(|registry| {
                registry.register(&term, tty_client(tty).map(|client| client.downgrade()))
            });
            term.borrow_mut().registration = Some(registration);
            return Ok(term);
        };
        log_debug(c"removing term %s", fmt_args![term.name.as_deref()]);
        Err(cause)
    }
}
/// Borrows the description installed by `tty_open` for capability lookups.
pub(crate) fn tty_term_of(tty: &tty) -> std::cell::Ref<'_, tty_term> {
    tty.term
        .as_ref()
        .expect("a tty being driven has a terminal")
        .borrow()
}

/// Borrows the terminal's capabilities exclusively when a tty has one.
pub(crate) fn tty_term_opt_mut(
    value: &mut Option<TerminalRef>,
) -> Option<std::cell::RefMut<'_, tty_term>> {
    value.as_ref().map(|term| term.borrow_mut())
}
pub(crate) unsafe fn tty_term_free(term: TerminalRef) {
    unsafe {
        log_debug(
            c"removing term %s",
            fmt_args![term.borrow().name.as_deref()],
        );
        drop(term);
    }
}
pub(crate) fn tty_term_read_list(
    name: &CStr,
    fd: core::ffi::c_int,
) -> Result<Vec<CString>, CString> {
    unsafe {
        let mut error: core::ffi::c_int = 0;
        if setupterm(name.as_ptr() as *mut core::ffi::c_char, fd, &raw mut error) != OK {
            let cause = match error {
                1 => format_alloc(c"can't use hardcopy terminal: %s", fmt_args![name]),
                0 => format_alloc(c"missing or unsuitable terminal: %s", fmt_args![name]),
                -1 => format_alloc(c"can't find terminfo database", fmt_args![]),
                _ => format_alloc(c"unknown error", fmt_args![]),
            };
            return Err(cause);
        }
        let mut caps: Vec<CString> = Vec::new();
        for ent in &tty_term_codes {
            let capability = match ent.type_0 {
                TTYCODE_NONE => continue,
                TTYCODE_STRING => {
                    let value = tigetstr(ent.name.as_ptr().cast_mut());
                    if value.is_null() || value == (-1_isize) as *mut core::ffi::c_char {
                        continue;
                    }
                    let value = CStr::from_ptr(value);
                    format_alloc(c"%s=%s", fmt_args![ent.name, value])
                }
                TTYCODE_NUMBER => {
                    let value = tigetnum(ent.name.as_ptr().cast_mut());
                    if value == -1 || value == -2 {
                        continue;
                    }
                    format_alloc(c"%s=%d", fmt_args![ent.name, value])
                }
                TTYCODE_FLAG => {
                    let value = tigetflag(ent.name.as_ptr().cast_mut());
                    if value == -1 {
                        continue;
                    }
                    format_alloc(
                        c"%s=%d",
                        fmt_args![ent.name, (value != 0) as core::ffi::c_int],
                    )
                }
                _ => fatalx(c"unknown capability type", fmt_args![]),
            };
            caps.push(capability);
        }
        del_curterm(cur_term);
        Ok(caps)
    }
}
fn tty_term_has(term: &tty_term, code: tty_code_code) -> core::ffi::c_int {
    !matches!(tty_term_code(term, code), TtyCode::None) as core::ffi::c_int
}
unsafe fn tty_term_string_i(term: &tty_term, code: tty_code_code, a: core::ffi::c_int) -> CString {
    unsafe {
        let x = term.string(code);
        let s: *const core::ffi::c_char =
            tiparm_s(1 as core::ffi::c_int, 0 as core::ffi::c_int, x.as_ptr(), a);
        if s.is_null() {
            log_debug(
                c"could not expand %s",
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
unsafe fn tty_term_string_ii(
    term: &tty_term,
    code: tty_code_code,
    a: core::ffi::c_int,
    b: core::ffi::c_int,
) -> CString {
    unsafe {
        let x = term.string(code);
        let s: *const core::ffi::c_char = tiparm_s(
            2 as core::ffi::c_int,
            0 as core::ffi::c_int,
            x.as_ptr(),
            a,
            b,
        );
        if s.is_null() {
            log_debug(
                c"could not expand %s",
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
unsafe fn tty_term_string_iii(
    term: &tty_term,
    code: tty_code_code,
    a: core::ffi::c_int,
    b: core::ffi::c_int,
    c: core::ffi::c_int,
) -> CString {
    unsafe {
        let x = term.string(code);
        let s: *const core::ffi::c_char = tiparm_s(
            3 as core::ffi::c_int,
            0 as core::ffi::c_int,
            x.as_ptr(),
            a,
            b,
            c,
        );
        if s.is_null() {
            log_debug(
                c"could not expand %s",
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
fn tty_term_string_s(term: &tty_term, code: tty_code_code, a: &CStr) -> CString {
    unsafe {
        let x = term.string(code);
        let s: *const core::ffi::c_char = tiparm_s(
            1 as core::ffi::c_int,
            1 as core::ffi::c_int,
            x.as_ptr(),
            a.as_ptr(),
        );
        if s.is_null() {
            log_debug(
                c"could not expand %s",
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
fn tty_term_string_ss(term: &tty_term, code: tty_code_code, a: &CStr, b: &CStr) -> CString {
    unsafe {
        let x = term.string(code);
        let s: *const core::ffi::c_char = tiparm_s(
            2 as core::ffi::c_int,
            3 as core::ffi::c_int,
            x.as_ptr(),
            a.as_ptr(),
            b.as_ptr(),
        );
        if s.is_null() {
            log_debug(
                c"could not expand %s",
                fmt_args![tty_term_codes[code as usize].name],
            );
            return CString::default();
        }
        CStr::from_ptr(s).to_owned()
    }
}
fn tty_term_flag(term: &tty_term, code: tty_code_code) -> core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as core::ffi::c_int;
    }
    let TtyCode::Flag(value) = tty_term_code(term, code) else {
        unsafe {
            fatalx(c"not a flag: %d", fmt_args![code as core::ffi::c_uint]);
        }
    };
    *value
}
/// How a capability reads: its number, its name and the value the terminal
/// gave it, as the caller's own string.
unsafe fn tty_term_describe(term: &tty_term, code: tty_code_code) -> CString {
    let name = tty_term_codes[code as usize].name;
    match tty_term_code(term, code) {
        TtyCode::None => format_alloc(
            c"%4u: %s: [missing]",
            fmt_args![code as core::ffi::c_uint, name],
        ),
        TtyCode::String(value) => {
            let out = strnvis(
                value,
                size_of::<[core::ffi::c_char; 128]>() as size_t,
                VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
            );
            let out = CString::new(out.output).expect("encoded capability has no nul");
            format_alloc(
                c"%4u: %s: (string) %s",
                fmt_args![code as core::ffi::c_uint, name, out.as_c_str()],
            )
        }
        TtyCode::Number(value) => format_alloc(
            c"%4u: %s: (number) %d",
            fmt_args![code as core::ffi::c_uint, name, *value],
        ),
        TtyCode::Flag(value) => format_alloc(
            c"%4u: %s: (flag) %s",
            fmt_args![
                code as core::ffi::c_uint,
                name,
                if *value != 0 { c"true" } else { c"false" }
            ],
        ),
    }
}

impl TerminalCapabilities for tty_term {
    fn new(name: &CStr) -> Self {
        Self {
            name: Some(name.to_owned()),
            features: 0,
            acs: [[0; 2]; 256],
            codes: (0..tty_term_ncodes()).map(|_| TtyCode::None).collect(),
            flags: 0,
            registration: None,
        }
    }

    fn name(&self) -> &CStr {
        self.name.as_deref().unwrap_or(c"")
    }

    fn capability_count(&self) -> u_int {
        tty_term_ncodes()
    }

    fn capability_name(&self, code: tty_code_code) -> &CStr {
        tty_term_codes[code as usize].name
    }

    fn capability(&self, code: tty_code_code) -> TerminalCapabilityRef<'_> {
        match tty_term_code(self, code) {
            TtyCode::None => TerminalCapabilityRef::Missing,
            TtyCode::String(value) => TerminalCapabilityRef::String(value),
            TtyCode::Number(value) => TerminalCapabilityRef::Number(*value),
            TtyCode::Flag(value) => TerminalCapabilityRef::Flag(*value != 0),
        }
    }

    fn apply_overrides(&mut self, capabilities: &CStr) {
        tty_term_apply(self, capabilities, 1);
    }

    fn apply_features(&mut self, features: core::ffi::c_int) -> bool {
        unsafe { tty_apply_features(self, features) != 0 }
    }

    fn refresh_derived(&mut self) {
        tty_term_refresh_derived(self);
    }

    fn flags(&self) -> core::ffi::c_int {
        self.flags
    }

    fn acs(&self, ch: u8) -> Option<&CStr> {
        let value = &self.acs[ch as usize];
        (value[0] != 0)
            .then(|| CStr::from_bytes_with_nul(value).expect("an ACS entry is terminated"))
    }

    fn expand_i(&self, code: tty_code_code, a: core::ffi::c_int) -> CString {
        unsafe { tty_term_string_i(self, code, a) }
    }

    fn expand_ii(&self, code: tty_code_code, a: core::ffi::c_int, b: core::ffi::c_int) -> CString {
        unsafe { tty_term_string_ii(self, code, a, b) }
    }

    fn expand_iii(
        &self,
        code: tty_code_code,
        a: core::ffi::c_int,
        b: core::ffi::c_int,
        c: core::ffi::c_int,
    ) -> CString {
        unsafe { tty_term_string_iii(self, code, a, b, c) }
    }

    fn expand_s(&self, code: tty_code_code, a: &CStr) -> CString {
        tty_term_string_s(self, code, a)
    }

    fn expand_ss(&self, code: tty_code_code, a: &CStr, b: &CStr) -> CString {
        tty_term_string_ss(self, code, a, b)
    }

    fn describe(&self, code: tty_code_code) -> CString {
        unsafe { tty_term_describe(self, code) }
    }
}

#[cfg(test)]
#[path = "../tests/test_terminal_registry.rs"]
mod registry_tests;

/// Describes terminal capabilities for a client's terminal, or all terminals when
/// the client has no terminal. The owned result remains usable during output.
///
/// # Safety
/// Exclude conflicting client/TTY access while resolving the terminal. The
/// registry snapshot performs no callbacks and retains no payload borrow.
pub(crate) unsafe fn tty_term_snapshots_for_client(
    client: Option<&ClientRef>,
) -> Vec<TerminalDescriptionSnapshot> {
    let terminal = client.and_then(|client| unsafe { client.as_tty().term.clone() });
    let terminal = terminal.as_ref().map(|terminal| terminal.borrow());
    tty_term_snapshots(terminal.as_deref())
}

#[cfg(test)]
pub use crate::consts::{TTYC_AX, TTYC_BEL, TTYC_COLORS};
