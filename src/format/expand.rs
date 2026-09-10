use crate::WindowPane;
use crate::args::argument_text::{ArgumentTextCodec as _, RustArgumentTextCodec};
use crate::cmd::cmdq_item;
use crate::format_modifier::FormatModifier;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::pane_identity::PaneIdentity;

use crate::window::WinlinkRef;
use crate::window_dimensions::WindowDimensionsState;
use crate::{UserAccount, UserAccountRecord};

use crate::cfg::configuration_files;
use crate::cmd::CmdqItemRef;
use crate::cmd::CmdqItemWeak;
use crate::cmd::{cmd_mouse_at, cmd_mouse_pane};

use crate::compat::strtonum;
use crate::environ::{EnvironmentStore, with_global_environment};
use crate::ffi::{
    __xpg_basename, ctime_r, dirname, fabs, fmod, fnmatch, gethostname, getpid, getuid, time,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_buf};
use crate::grid::{Grid, Hyperlinks, grid_view_get_cell};
use crate::grid::{grid_default_cell, grid_get_line_ref, grid_peek_line};
use crate::job::{job_free, job_run};

use crate::log::{log_debug, log_get_level};
use crate::modes::{window_copy_get_hyperlink, window_copy_get_line, window_copy_get_word};
use crate::names::{RustWindowNameParser, WindowNameParser};

use crate::osdep_linux::{osdep_get_cwd, osdep_get_name};
use crate::pane_command::PaneCommandState;
use crate::pane_exit::PaneExitState;
use crate::pane_geometry::PaneGeometryState;
use crate::pane_search::PaneSearchState;
use crate::paste::{PasteBufferStore, with_paste_buffers};

use crate::regsub::{RegsubEngine, RustRegsub};
use crate::screen::Screen;
use crate::server::client_ref_of;
use crate::server::marked_pane;
use crate::server::server_check_marked;
use crate::server::server_client_get_cwd;
use crate::server::{client_walk, with_clients};

use crate::session::{
    SESSION_GROUPS, SESSIONS_FIELD, next_session_id, session_group_attached_count,
    session_group_count, session_group_name,
};

pub use crate::consts::{
    ALL_MOUSE_MODES, CLIENT_CONTROL, CLIENT_READONLY, CLIENT_UTF8, EXTENDED_KEY_MODES,
    FNM_CASEFOLD, FORMAT_FORCE, FORMAT_NOJOBS, FORMAT_NONE, FORMAT_PANE, FORMAT_STATUS,
    FORMAT_VERBOSE, FORMAT_WINDOW, GRID_FLAG_PADDING, GRID_FLAG_TAB, GRID_LINE_WRAPPED, JOB_NOWAIT,
    MODE_BRACKETPASTE, MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_VERY_VISIBLE, MODE_INSERT,
    MODE_KCURSOR, MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2, MODE_KKEYPAD, MODE_MOUSE_ALL,
    MODE_MOUSE_BUTTON, MODE_MOUSE_SGR, MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8, MODE_ORIGIN,
    MODE_SYNC, MODE_WRAP, PANE_INPUTOFF, PANE_STATUS_BOTTOM, PANE_STATUS_TOP, PANE_STATUSDRAWN,
    PANE_STATUSREADY, PANE_UNSEENCHANGES, PANE_ZOOMED, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, REG_EXTENDED, REG_ICASE,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_UNDERLINE, SORT_ACTIVITY, SORT_CREATION,
    SORT_INDEX, SORT_NAME, SORT_ORDER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, TTY_STARTED, WINDOW_PANE_NO_MODE, WINDOW_ZOOMED,
    WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
};
use crate::sort::{SortCriteria, sort_get_clients, sort_get_sessions};
use crate::style::{ColourEngine, RustColourEngine};
use crate::terminfo::{RustTerminalFeatureSet, TerminalFeatureSet};
use crate::text::{RustUtf8VisModel, Utf8VisModel, utf8_cstrhas, utf8_set, utf8_vec_tocstr};
use crate::tmux::{get_timer, getversion, sig2name};
use crate::tmux::{global_options, global_s_options, global_w_options, socket_path, start_time};
use crate::tree::GlobalTree;
use crate::tty::{tty_default_colours, tty_window_offset};
pub use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::{
    window_count_panes, window_pane_index, window_pane_is_floating, window_pane_mode,
    window_pane_printable_flags, window_pane_search, window_pane_zindex, window_printable_flags,
    winlink_count,
};
use crate::xmalloc::xasprintf;
use crate::{CommandTextCodec, RustCommandTextCodec};
use crate::{FormatText, RustFormatText};
use ::core::ffi::CStr;
use ::std::ffi::CString;

#[derive(Default)]
#[repr(C)]
pub struct format_tree {
    pub type_0: format_type,
    /// The client the tree draws its client formats from, observed rather
    /// than held. This is not the client the tree was created for — that one
    /// is `client_ref` below — but the one `format_defaults` picked out.
    pub(crate) c: Option<ClientWeak>,
    /// The session the tree draws on, observed rather than held.
    pub(crate) s: Option<SessionWeak>,
    /// The link the tree draws on, named by the session that holds it and
    /// the index it holds it at, or nothing when it draws on none.
    pub(crate) wl: Option<(SessionWeak, core::ffi::c_int)>,
    /// The window the tree draws on, observed the same way.
    pub(crate) w: Option<WindowWeak>,
    /// The pane the tree draws on, observed independently of its window.
    pub wp: Option<RustWindowPaneWeak>,
    /// The name of the buffer the tree draws on, or nothing when it draws on
    /// none. A buffer is named by its name and nothing else.
    pub pb: Option<CString>,
    /// The queue item the tree was made for, observed rather than held.
    pub(crate) item: Option<CmdqItemWeak>,
    pub(crate) client: Option<ClientRef>,
    pub flags: core::ffi::c_int,
    pub tag: u_int,
    pub m: mouse_event,
    pub tree: format_entry_tree,
}

impl format_tree {
    /// The session the tree draws on, or null when it draws on none or the
    /// server has since given it up.
    pub(crate) fn session(&self) -> Option<SessionRef> {
        self.s.as_ref().and_then(SessionWeak::upgrade)
    }

    /// Records `s` as the session the tree draws on.
    pub(crate) fn set_session(&mut self, s: Option<&session>) {
        self.s = s
            .and_then(crate::session::session_ref_of)
            .map(|s| s.downgrade());
    }

    /// The link the tree draws on, retained through the session that owns it.
    pub(crate) fn winlink(&self) -> Option<WinlinkRef> {
        let (held, idx) = self.wl.as_ref()?;
        let held = held.upgrade()?;
        WinlinkRef::new(held, *idx)
    }

    /// Records `wl` as the link the tree draws on.
    pub(crate) fn set_winlink(&mut self, wl: Option<&winlink>) {
        self.wl = wl.and_then(|wl| wl.session().map(|s| (s.downgrade(), wl.idx)));
    }

    /// The window the tree draws on, or null the same way.
    pub(crate) fn window(&self) -> Option<WindowRef> {
        self.w.as_ref().and_then(WindowWeak::upgrade)
    }

    /// Records `w` as the window the tree draws on.
    pub(crate) fn set_window(&mut self, w: Option<&WindowRef>) {
        self.w = w.map(WindowRef::downgrade);
    }

    /// Observes the pane independently of changes to its window membership.
    pub(crate) fn pane_handle(&self) -> Option<crate::window::RustWindowPaneWeak> {
        self.wp.as_ref().filter(|pane| pane.is_alive()).cloned()
    }

    /// Records `wp` as the pane the tree draws on.
    pub(crate) fn set_pane(&mut self, wp: Option<&impl crate::WindowPane>) {
        self.wp = wp.and_then(|wp| crate::window::window_pane_ref_of(wp));
    }

    /// The client the tree draws its client formats from, or null when it
    /// draws on none or the server has since given it up.
    pub(crate) fn drawn_client(&self) -> Option<ClientRef> {
        self.c.as_ref().and_then(ClientWeak::upgrade)
    }

    /// Records `c` as the client the tree draws its client formats from.
    pub(crate) fn set_drawn_client(&mut self, c: Option<&ClientRef>) {
        self.c = c.map(ClientRef::downgrade);
    }

    /// The queue item the tree was made for, or null when it was made for
    /// none or the queue has since given it up.
    pub(crate) fn item(&self) -> Option<CmdqItemRef> {
        self.item.as_ref().and_then(CmdqItemWeak::upgrade)
    }

    /// Records `item` as the item the tree was made for.
    pub(crate) fn set_item(&mut self, item: Option<&cmdq_item>) {
        self.item = item
            .and_then(crate::cmd::cmdq_item_ref_of)
            .map(|item| item.downgrade());
    }

    /// The name of the buffer the tree draws on, if it has one.
    pub(crate) fn buffer_name(&self) -> Option<&CStr> {
        self.pb.as_deref()
    }

    /// Records the buffer name the tree draws on.
    pub(crate) fn set_buffer(&mut self, name: Option<&CStr>) {
        self.pb = name.map(CStr::to_owned);
    }

    /// The client whose jobs and working directory this tree draws on, if it
    /// was created with one.
    pub(crate) fn client(&self) -> Option<ClientRef> {
        self.client.clone()
    }
}
/// The entries of a format tree, by key. An entry lives in the map, so a
/// pointer to one lasts only until the same tree is added to again.
pub type format_entry_tree = std::collections::BTreeMap<CString, format_entry>;
#[repr(C)]
pub struct format_entry {
    pub value: Option<CString>,
    pub time: time_t,
    pub cb: format_entry_cb,
}
pub type format_type = core::ffi::c_uint;
pub const FORMAT_TYPE_PANE: format_type = 3;
pub const FORMAT_TYPE_WINDOW: format_type = 2;
pub const FORMAT_TYPE_SESSION: format_type = 1;
pub const FORMAT_TYPE_UNKNOWN: format_type = 0;

/// The key a job hangs under: the tree's tag, then the command.
pub type format_job_key = (u_int, CString);

/// The running jobs of one client, or of the server.
pub type format_job_tree = std::collections::BTreeMap<format_job_key, Box<format_job>>;

#[derive(Clone)]
struct FormatJobLocator {
    client: Option<ClientWeak>,
    key: format_job_key,
}
#[repr(C)]
pub struct format_job {
    pub(crate) client: Option<ClientWeak>,
    pub tag: u_int,
    pub cmd: CString,
    pub expanded: Option<CString>,
    pub last: time_t,
    pub out: Option<CString>,
    pub updated: core::ffi::c_int,
    /// The id of the job the entry is running, or nothing while it runs
    /// none. A job is named by its id and nothing else, so the entry never
    /// names one that has finished.
    pub job: Option<u_int>,
    pub status: core::ffi::c_int,
}

/// Cloneable recursion metadata for one format expansion.
///
/// The format tree is borrowed separately for each expansion call. A nested
/// expansion clones these limits, flags and retained timezone while reborrowing its tree.
#[derive(Clone, Default)]
#[repr(C)]
pub struct format_expand_state {
    pub r#loop: u_int,
    pub start_time: uint64_t,
    pub flags: core::ffi::c_int,
    pub time: time_t,
    pub tm: tm,
}
#[derive(Copy, Clone)]
pub enum FormatTableCallback {
    String(unsafe fn(&format_tree) -> Option<CString>),
    Time(unsafe fn(&format_tree) -> Option<timeval>),
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_table_entry {
    pub key: &'static CStr,
    pub cb: FormatTableCallback,
}

pub const LESS_THAN_EQUAL: format_operator = 10;
pub const LESS_THAN: format_operator = 9;
pub const GREATER_THAN_EQUAL: format_operator = 8;
pub const GREATER_THAN: format_operator = 7;
pub const NOT_EQUAL: format_operator = 6;
pub const EQUAL: format_operator = 5;
pub const MODULUS: format_operator = 4;
pub const DIVIDE: format_operator = 3;
pub const MULTIPLY: format_operator = 2;
pub const SUBTRACT: format_operator = 1;
pub const ADD: format_operator = 0;
pub type format_operator = core::ffi::c_uint;

pub const REG_NOSUB: core::ffi::c_int = (1 as core::ffi::c_int) << 3 as core::ffi::c_int;
pub const INT64_MAX: core::ffi::c_long = 9223372036854775807 as core::ffi::c_long;

pub const FORMAT_LAST: core::ffi::c_int = 0x10 as core::ffi::c_int;

const format_jobs_FIELD: crate::server_state::LocalField<
    std::rc::Rc<GlobalTree<format_job_key, Box<format_job>>>,
> = crate::server_state::LocalField::new(|state| &state.format_jobs);
pub const FORMAT_MAX_WIDTH: core::ffi::c_int = 10000 as core::ffi::c_int;
pub const FORMAT_MAX_REPEAT: core::ffi::c_int = 10000 as core::ffi::c_int;
pub const FORMAT_MAX_PRECISION: core::ffi::c_int = 100 as core::ffi::c_int;
pub const FORMAT_TIMESTRING: core::ffi::c_int = 0x1 as core::ffi::c_int;
pub const FORMAT_BASENAME: core::ffi::c_int = 0x2 as core::ffi::c_int;
pub const FORMAT_DIRNAME: core::ffi::c_int = 0x4 as core::ffi::c_int;
pub const FORMAT_QUOTE_SHELL: core::ffi::c_int = 0x8 as core::ffi::c_int;
pub const FORMAT_LITERAL: core::ffi::c_int = 0x10 as core::ffi::c_int;
pub const FORMAT_EXPAND: core::ffi::c_int = 0x20 as core::ffi::c_int;
pub const FORMAT_EXPANDTIME: core::ffi::c_int = 0x40 as core::ffi::c_int;
pub const FORMAT_SESSIONS: core::ffi::c_int = 0x80 as core::ffi::c_int;
pub const FORMAT_WINDOWS: core::ffi::c_int = 0x100 as core::ffi::c_int;
pub const FORMAT_PANES: core::ffi::c_int = 0x200 as core::ffi::c_int;
pub const FORMAT_PRETTY: core::ffi::c_int = 0x400 as core::ffi::c_int;
pub const FORMAT_LENGTH: core::ffi::c_int = 0x800 as core::ffi::c_int;
pub const FORMAT_WIDTH: core::ffi::c_int = 0x1000 as core::ffi::c_int;
pub const FORMAT_QUOTE_STYLE: core::ffi::c_int = 0x2000 as core::ffi::c_int;
pub const FORMAT_WINDOW_NAME: core::ffi::c_int = 0x4000 as core::ffi::c_int;
pub const FORMAT_SESSION_NAME: core::ffi::c_int = 0x8000 as core::ffi::c_int;
pub const FORMAT_CHARACTER: core::ffi::c_int = 0x10000 as core::ffi::c_int;
pub const FORMAT_COLOUR: core::ffi::c_int = 0x20000 as core::ffi::c_int;
pub const FORMAT_CLIENTS: core::ffi::c_int = 0x40000 as core::ffi::c_int;
pub const FORMAT_NOT: core::ffi::c_int = 0x80000 as core::ffi::c_int;
pub const FORMAT_NOT_NOT: core::ffi::c_int = 0x100000 as core::ffi::c_int;
pub const FORMAT_REPEAT: core::ffi::c_int = 0x200000 as core::ffi::c_int;
pub const FORMAT_QUOTE_ARGUMENTS: core::ffi::c_int = 0x400000 as core::ffi::c_int;
pub const FORMAT_LOOP_LIMIT: core::ffi::c_int = 100 as core::ffi::c_int;
pub const FORMAT_TIME_LIMIT: core::ffi::c_int = 100 as core::ffi::c_int;
pub const FORMAT_EXPAND_TIME: core::ffi::c_int = 0x1 as core::ffi::c_int;
pub const FORMAT_EXPAND_NOJOBS: core::ffi::c_int = 0x2 as core::ffi::c_int;
static format_upper: [Option<&CStr>; 26] = [
    None,
    None,
    None,
    Some(c"pane_id"),
    None,
    Some(c"window_flags"),
    None,
    Some(c"host"),
    Some(c"window_index"),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(c"pane_index"),
    None,
    None,
    Some(c"session_name"),
    Some(c"pane_title"),
    None,
    None,
    Some(c"window_name"),
    None,
    None,
    None,
];
static format_lower: [Option<&CStr>; 26] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(c"host_short"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];
#[inline]
fn format_logging(ft: &mut format_tree) -> core::ffi::c_int {
    (log_get_level() != 0 as core::ffi::c_int || ft.flags & FORMAT_VERBOSE != 0) as core::ffi::c_int
}
unsafe fn format_log1(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    from: &CStr,
    fmt: &CStr,
    args: &[FmtArg],
) {
    unsafe {
        if format_logging(&mut *ft) == 0 {
            return;
        }
        let s = format_alloc(fmt, args);
        log_debug(c"%s: %s", fmt_args![from, s.as_c_str()]);
        if let Some(item) = (*ft).item()
            && ft.flags & FORMAT_VERBOSE != 0
        {
            item.with_item(|item| {
                item.print(
                    c"#%.*s%s",
                    fmt_args![es.r#loop, c"          ", s.as_c_str()],
                );
            });
        }
    }
}
unsafe fn with_format_job<R>(
    locator: &FormatJobLocator,
    f: impl FnOnce(&mut format_job) -> R,
) -> Option<R> {
    let format_jobs = format_jobs_FIELD.get();

    match locator.client.as_ref() {
        Some(watched) => {
            let mut client = watched.upgrade()?;
            unsafe { client.with_format_jobs(|jobs| jobs.get_mut(&locator.key).map(|fj| f(fj)))? }
        }
        None => {
            let mut jobs = format_jobs.map();
            jobs.get_mut(&locator.key).map(|fj| f(fj))
        }
    }
}

unsafe fn format_job_update(job: JobEvent, locator: FormatJobLocator) {
    unsafe {
        let event = job.event;
        let mut line = None;

        loop {
            let Some(next) = event.with_input(|buffer| buffer.read_line()).flatten() else {
                break;
            };
            line = Some(next);
        }
        let Some(line) = line else {
            return;
        };
        let t: time_t = time(core::ptr::null_mut::<time_t>());
        let notify = with_format_job(&locator, |fj| {
            fj.updated = 1 as core::ffi::c_int;
            fj.out = Some(CString::from_vec_unchecked(line.to_vec()));
            log_debug(
                c"%s: %s: %s",
                fmt_args![c"format_job_update", fj.cmd.as_c_str(), fj.out.as_deref()],
            );
            if fj.status != 0 && fj.last != t {
                fj.last = t;
                true
            } else {
                false
            }
        });
        if notify == Some(true)
            && let Some(mut c) = locator.client.as_ref().and_then(ClientWeak::upgrade)
        {
            c.request_status_redraw();
        }
    }
}
unsafe fn format_job_complete(job: JobEvent, locator: FormatJobLocator) {
    unsafe {
        let event = job.event;
        let bytes = event
            .with_input(|buffer| {
                buffer
                    .read_line()
                    .map_or_else(|| buffer.as_slice().to_vec(), |line| line.to_vec())
            })
            .unwrap_or_default();
        let notify = with_format_job(&locator, |fj| {
            fj.job = None;
            log_debug(
                c"%s: %s: %s",
                fmt_args![c"format_job_complete", fj.cmd.as_c_str(), bytes.as_slice()],
            );
            if !bytes.is_empty() || fj.updated == 0 {
                fj.out = Some(CString::from_vec_unchecked(bytes));
            }
            if fj.status != 0 {
                fj.status = 0 as core::ffi::c_int;
                true
            } else {
                false
            }
        });
        if notify == Some(true)
            && let Some(mut c) = locator.client.as_ref().and_then(ClientWeak::upgrade)
        {
            c.request_status_redraw();
        }
    }
}
fn format_list_value(buffer: &mut ByteBuffer) -> Option<CString> {
    if buffer.is_empty() {
        return None;
    }
    let data = buffer.as_slice();
    let end = data
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(data.len());
    Some(CString::new(&data[..end]).expect("the value ends before its first NUL"))
}
unsafe fn format_job_get(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    cmd: &CStr,
) -> CString {
    let format_jobs = format_jobs_FIELD.get();

    unsafe {
        let mut client = ft.client();
        let locator = FormatJobLocator {
            client: client.as_ref().map(ClientRef::downgrade),
            key: (ft.tag, cmd.to_owned()),
        };
        let entry = || {
            Box::new(format_job {
                client: locator.client.clone(),
                tag: ft.tag,
                cmd: cmd.to_owned(),
                expanded: None,
                last: 0,
                out: None,
                updated: 0,
                job: None,
                status: 0,
            })
        };
        if let Some(client) = client.as_mut() {
            client.ensure_format_job(locator.key.clone(), entry);
        } else {
            format_jobs
                .map()
                .entry(locator.key.clone())
                .or_insert_with(entry);
        }
        let mut next = es.clone();
        next.flags |= FORMAT_EXPAND_NOJOBS;
        next.flags &= !FORMAT_EXPAND_TIME;
        let expanded = format_expand1(ft, &mut next, cmd);
        let t = time(core::ptr::null_mut::<time_t>());
        let Some((old_job, start_job)) = with_format_job(&locator, |fj| {
            let force = if fj.expanded.as_deref() != Some(expanded.as_c_str()) {
                fj.expanded = Some(expanded.clone());
                true
            } else {
                ft.flags & FORMAT_FORCE != 0
            };
            let old_job = if force { fj.job.take() } else { None };
            (old_job, force || fj.job.is_none() && fj.last != t)
        }) else {
            return CString::default();
        };
        if let Some(id) = old_job {
            job_free(id);
        }
        if start_job {
            let update_locator = locator.clone();
            let complete_locator = locator.clone();
            let cwd = match client.as_ref() {
                Some(client) => client.working_directory(),
                None => server_client_get_cwd(None, None),
            };
            let id = job_run(
                Some(&expanded),
                &[],
                None,
                None,
                Some(cwd.as_c_str()),
                Some(std::rc::Rc::new(move |job| {
                    format_job_update(job, update_locator.clone())
                })),
                Some(Box::new(move |job| {
                    format_job_complete(job, complete_locator.clone())
                })),
                JOB_NOWAIT,
                -1,
                -1,
            );
            if with_format_job(&locator, |fj| {
                fj.job = id;
                if id.is_none() {
                    fj.out = Some(format_alloc(
                        c"<'%s' didn't start>",
                        fmt_args![fj.cmd.as_c_str()],
                    ));
                }
                fj.last = t;
                fj.updated = 0;
            })
            .is_none()
            {
                if let Some(id) = id {
                    job_free(id);
                }
                return CString::default();
            }
        } else {
            with_format_job(&locator, |fj| {
                if fj.job.is_some() && t - fj.last > 1 && fj.out.is_none() {
                    fj.out = Some(format_alloc(
                        c"<'%s' not ready>",
                        fmt_args![fj.cmd.as_c_str()],
                    ));
                }
            });
        }
        let output = with_format_job(&locator, |fj| {
            if ft.flags & FORMAT_STATUS != 0 {
                fj.status = 1;
            }
            fj.out.clone()
        })
        .flatten();
        output.map_or_else(CString::default, |output| {
            format_expand1(ft, &mut next, &output)
        })
    }
}
unsafe fn format_job_tidy(jobs: &mut format_job_tree, force: core::ffi::c_int) {
    unsafe {
        let now: time_t = time(core::ptr::null_mut::<time_t>());
        jobs.retain(|_, fj| {
            if force == 0 && (fj.last > now || now - fj.last < 3600) {
                return true;
            }
            log_debug(c"%s: %s", fmt_args![c"format_job_tidy", fj.cmd.as_c_str()]);
            if let Some(id) = fj.job.take() {
                job_free(id);
            }
            false
        });
    }
}
/// `fmt` through `strftime` for `tm`, or nothing when what it spells out
/// would not fit in `max` bytes counting the terminator, which is the case
/// strftime reports by answering zero.
fn format_strftime(max: size_t, fmt: &CStr, tm: &tm) -> Option<CString> {
    unsafe {
        let mut buf = std::vec::from_elem(0u8, max);
        let used = tm.format_into(&mut buf, fmt);
        if used == 0 as size_t {
            return None;
        }
        buf.truncate(used as usize);
        CString::new(buf).ok()
    }
}
pub fn format_tidy_jobs() {
    let format_jobs = format_jobs_FIELD.get();

    unsafe {
        format_job_tidy(&mut format_jobs.map(), 0 as core::ffi::c_int);
        for mut c in client_walk() {
            c.with_format_jobs(|jobs| format_job_tidy(jobs, 0 as core::ffi::c_int));
        }
    }
}
pub unsafe fn format_free_jobs(jobs: Option<Box<format_job_tree>>) {
    unsafe {
        if let Some(mut jobs) = jobs {
            format_job_tidy(&mut jobs, 1 as core::ffi::c_int);
        }
    }
}
fn format_printf(fmt: &CStr, args: &[FmtArg]) -> CString {
    format_alloc(fmt, args)
}
fn format_callback_copy(value: &CStr) -> CString {
    value.to_owned()
}
fn format_cb_host(_ft: &format_tree) -> Option<CString> {
    let mut host = [0u8; 65];
    if unsafe { gethostname(host.as_mut_ptr().cast(), host.len()) } != 0 {
        return Some(format_callback_copy(c""));
    }
    Some(format_callback_copy(
        CStr::from_bytes_until_nul(&host).ok()?,
    ))
}
fn format_cb_host_short(ft: &format_tree) -> Option<CString> {
    let mut host = format_cb_host(ft)?.into_bytes();
    if let Some(dot) = host.iter().position(|&byte| byte == b'.') {
        host.truncate(dot);
    }
    Some(CString::new(host).expect("host name has no NUL"))
}
fn format_cb_pid(_ft: &format_tree) -> Option<CString> {
    let value = xasprintf(c"%ld", fmt_args![unsafe { getpid() } as core::ffi::c_long]);
    Some(value)
}
unsafe fn format_cb_session_attached_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let s = ft.session()?;
        let mut buffer = ByteBuffer::new();
        with_clients(|clients| {
            for loop_0 in clients {
                if loop_0
                    .attached_session()
                    .is_some_and(|attached| s.ptr_eq(&attached))
                {
                    if !buffer.is_empty() {
                        buffer.append(b",");
                    }
                    format_buf(&mut buffer, c"%s", fmt_args![loop_0.name()]);
                }
            }
        });
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_session_alert(ft: &format_tree) -> Option<CString> {
    unsafe {
        let owner = ft.session()?;
        let s = owner.as_session();
        let mut alerts: Vec<u8> = Vec::new();
        let mut alerted: core::ffi::c_int = 0 as core::ffi::c_int;
        for wl in s.windows.values() {
            if !(wl.flags & WINLINK_ALERTFLAGS == 0 as core::ffi::c_int) {
                if !alerted & wl.flags & WINLINK_ACTIVITY != 0 {
                    alerts.push(b'#');
                    alerted |= WINLINK_ACTIVITY;
                }
                if !alerted & wl.flags & WINLINK_BELL != 0 {
                    alerts.push(b'!');
                    alerted |= WINLINK_BELL;
                }
                if !alerted & wl.flags & WINLINK_SILENCE != 0 {
                    alerts.push(b'~');
                    alerted |= WINLINK_SILENCE;
                }
            }
        }
        Some(CString::new(alerts).expect("alert marks have no NUL"))
    }
}
unsafe fn format_cb_session_alerts(ft: &format_tree) -> Option<CString> {
    unsafe {
        let owner = ft.session()?;
        let s = owner.as_session();
        let mut alerts: Vec<u8> = Vec::new();
        for wl in s.windows.values() {
            if !(wl.flags & WINLINK_ALERTFLAGS == 0 as core::ffi::c_int) {
                if !alerts.is_empty() {
                    alerts.push(b',');
                }
                alerts.extend_from_slice(format!("{}", wl.idx).as_bytes());
                if wl.flags & WINLINK_ACTIVITY != 0 {
                    alerts.push(b'#');
                }
                if wl.flags & WINLINK_BELL != 0 {
                    alerts.push(b'!');
                }
                if wl.flags & WINLINK_SILENCE != 0 {
                    alerts.push(b'~');
                }
            }
        }
        alerts.truncate(1023);
        Some(CString::new(alerts).expect("window numbers and marks have no NUL"))
    }
}
unsafe fn format_cb_session_stack(ft: &format_tree) -> Option<CString> {
    unsafe {
        let owner = ft.session()?;
        let s = owner.as_session();
        let mut result: Vec<u8> = format!("{}", s.curw()?.idx).into_bytes();
        for idx in &s.lastw {
            if !result.is_empty() {
                result.push(b',');
            }
            result.extend_from_slice(format!("{}", idx).as_bytes());
        }
        result.truncate(1023);
        Some(CString::new(result).expect("window numbers have no NUL"))
    }
}
unsafe fn format_cb_window_stack_index(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    let session = link.session()?;
    let index = unsafe { session.as_session() }
        .lastw
        .iter()
        .position(|idx| *idx == link.idx)
        .map_or(0, |index| (index as u_int).wrapping_add(1));
    Some(format_printf(c"%u", fmt_args![index]))
}
fn format_cb_window_linked_sessions_list(ft: &format_tree) -> Option<CString> {
    let mut buffer = ByteBuffer::new();
    let link = ft.winlink()?;
    let window = link.get()?.window_handle()?.clone();
    for held in window.winlinks() {
        let Some(_link) = held.get() else {
            continue;
        };
        if !buffer.is_empty() {
            buffer.append(b",");
        }
        format_buf(
            &mut buffer,
            c"%s",
            fmt_args![held.session().name().as_deref()],
        );
    }
    format_list_value(&mut buffer)
}
fn format_cb_window_active_sessions(ft: &format_tree) -> Option<CString> {
    let mut n: u_int = 0 as u_int;
    let link = ft.winlink()?;
    let window = link.get()?.window_handle()?.clone();
    for held in window.winlinks() {
        let Some(wl) = held.get() else {
            continue;
        };
        if held
            .session()
            .curw()
            .is_some_and(|current| current.index() == wl.idx)
        {
            n = n.wrapping_add(1);
        }
    }
    let value = xasprintf(c"%u", fmt_args![n]);
    Some(value)
}
fn format_cb_window_active_sessions_list(ft: &format_tree) -> Option<CString> {
    let mut buffer = ByteBuffer::new();
    let link = ft.winlink()?;
    let window = link.get()?.window_handle()?.clone();
    for held in window.winlinks() {
        let Some(wl) = held.get() else {
            continue;
        };
        if held
            .session()
            .curw()
            .is_some_and(|current| current.index() == wl.idx)
        {
            if !buffer.is_empty() {
                buffer.append(b",");
            }
            format_buf(
                &mut buffer,
                c"%s",
                fmt_args![held.session().name().as_deref()],
            );
        }
    }
    format_list_value(&mut buffer)
}
fn format_cb_window_active_clients(ft: &format_tree) -> Option<CString> {
    {
        let mut n: u_int = 0 as u_int;
        let link = ft.winlink()?;
        let window = link.get()?.window_handle()?.clone();
        with_clients(|clients| {
            for loop_0 in clients {
                let session = loop_0.attached_session();
                if session
                    .as_ref()
                    .and_then(|session| session.curw())
                    .and_then(|link| link.window())
                    .is_some_and(|current| current.ptr_eq(&window))
                {
                    n = n.wrapping_add(1);
                }
            }
        });
        let value = xasprintf(c"%u", fmt_args![n]);
        Some(value)
    }
}
unsafe fn format_cb_window_active_clients_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut buffer = ByteBuffer::new();
        let link = ft.winlink()?;
        let window = link.get()?.window_handle()?.clone();
        with_clients(|clients| {
            for loop_0 in clients {
                let session = loop_0.attached_session();
                if session
                    .as_ref()
                    .and_then(|session| session.curw())
                    .and_then(|link| link.window())
                    .is_some_and(|current| current.ptr_eq(&window))
                {
                    if !buffer.is_empty() {
                        buffer.append(b",");
                    }
                    format_buf(&mut buffer, c"%s", fmt_args![loop_0.name()]);
                }
            }
        });
        format_list_value(&mut buffer)
    }
}
fn format_cb_window_layout(ft: &format_tree) -> Option<CString> {
    let owner = ft.window()?;
    let w = owner.as_window();
    owner.dump_layout_cell(w.saved_layout_root.as_deref().or(w.layout_root.as_deref()))
}
fn format_cb_window_visible_layout(ft: &format_tree) -> Option<CString> {
    let owner = ft.window()?;
    let w = owner.as_window();
    owner.dump_layout_cell(w.layout_root.as_deref())
}
unsafe fn format_cb_start_command(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let command = RustCommandTextCodec.stringify(&wp.pane_command().argv);
        Some(format_callback_copy(&command))
    }
}
unsafe fn format_cb_start_path(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let command = wp.pane_command();
        if command.cwd.is_none() {
            return Some(format_callback_copy(c""));
        }
        Some(format_callback_copy(command.cwd.as_deref().unwrap_or(c"")))
    }
}
unsafe fn format_cb_current_command(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let pane_command = wp.pane_command();
        pane_command.shell.as_ref()?;
        let cmd = osdep_get_name(*wp.fd()).filter(|cmd| !cmd.as_bytes().is_empty());
        let Some(cmd) = cmd else {
            let command = RustCommandTextCodec.stringify(&pane_command.argv);
            if command.as_bytes().is_empty() {
                let value = RustWindowNameParser
                    .parse_window_name(pane_command.shell.as_deref().unwrap_or(c""));
                return Some(format_callback_copy(&value));
            }
            let value = RustWindowNameParser.parse_window_name(&command);
            return Some(format_callback_copy(&value));
        };
        let value = RustWindowNameParser.parse_window_name(&cmd);
        Some(format_callback_copy(&value))
    }
}
unsafe fn format_cb_current_path(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let cwd = osdep_get_cwd(*wp.fd())?;
        Some(format_callback_copy(&cwd))
    }
}
unsafe fn format_cb_history_bytes(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let mut size: size_t = 0 as size_t;
        let mut i: u_int;
        let gd = RustScreen::grid(wp.base());
        i = 0 as u_int;
        while i < gd.hsize.wrapping_add(gd.sy) {
            let gl = grid_get_line_ref(gd, i);
            size = (size as core::ffi::c_ulong).wrapping_add(
                ((*gl).cellsize() as usize).wrapping_mul(size_of::<grid_cell_entry>())
                    as core::ffi::c_ulong,
            ) as size_t as size_t;
            size = (size as core::ffi::c_ulong).wrapping_add(
                ((*gl).extdsize() as usize).wrapping_mul(size_of::<grid_extd_entry>())
                    as core::ffi::c_ulong,
            ) as size_t as size_t;
            i = i.wrapping_add(1);
        }
        size = (size as core::ffi::c_ulong).wrapping_add(
            (gd.hsize.wrapping_add(gd.sy) as usize).wrapping_mul(size_of::<grid_line>())
                as core::ffi::c_ulong,
        ) as size_t as size_t;
        let value = xasprintf(c"%zu", fmt_args![size]);
        Some(value)
    }
}
unsafe fn format_cb_history_all_bytes(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let mut i: u_int;

        let mut cells: u_int = 0 as u_int;
        let mut extended_cells: u_int = 0 as u_int;
        let gd = RustScreen::grid(wp.base());
        let lines: u_int = gd.hsize.wrapping_add(gd.sy);
        i = 0 as u_int;
        while i < lines {
            let gl = grid_get_line_ref(gd, i);
            cells = cells.wrapping_add((*gl).cellsize());
            extended_cells = extended_cells.wrapping_add((*gl).extdsize());
            i = i.wrapping_add(1);
        }
        let value = xasprintf(
            c"%u,%zu,%u,%zu,%u,%zu",
            fmt_args![
                lines,
                (lines as usize).wrapping_mul(size_of::<grid_line>()),
                cells,
                (cells as usize).wrapping_mul(size_of::<grid_cell_entry>()),
                extended_cells,
                (extended_cells as usize).wrapping_mul(size_of::<grid_extd_entry>())
            ],
        );
        Some(value)
    }
}
unsafe fn format_cb_pane_tabs(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let mut buffer = ByteBuffer::new();
        let mut i: u_int;
        i = 0 as u_int;
        while i < RustScreen::grid(wp.base()).sx {
            if wp.base().tab_is_set(i) {
                if !buffer.is_empty() {
                    buffer.append(b",");
                }
                format_buf(&mut buffer, c"%u", fmt_args![i]);
            }
            i = i.wrapping_add(1);
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_pane_fg(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut pane = ft.pane_handle()?;
        let wp = pane.get_mut()?;
        let mut gc = grid_default_cell;
        tty_default_colours(&mut gc, wp);
        Some(RustColourEngine.to_string(gc.fg))
    }
}
unsafe fn format_cb_pane_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        window_pane_printable_flags(&pane)
    }
}
fn format_cb_pane_floating_flag(ft: &format_tree) -> Option<CString> {
    let pane = ft.pane_handle()?;
    let window = pane.window()?;
    Some(format_callback_copy(
        if window_pane_is_floating(&window.as_window(), &pane) != 0 {
            c"1"
        } else {
            c"0"
        },
    ))
}
unsafe fn format_cb_pane_bg(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut pane = ft.pane_handle()?;
        let wp = pane.get_mut()?;
        let mut gc = grid_default_cell;
        tty_default_colours(&mut gc, wp);
        Some(RustColourEngine.to_string(gc.bg))
    }
}
fn format_cb_session_group_list(ft: &format_tree) -> Option<CString> {
    let s = ft.session()?;
    s.with_group(|group| {
        let mut buffer = ByteBuffer::new();
        for member in group.sessions.iter().filter_map(SessionWeak::upgrade) {
            if !buffer.is_empty() {
                buffer.append(b",");
            }
            format_buf(&mut buffer, c"%s", fmt_args![member.name().as_deref()]);
        }
        format_list_value(&mut buffer)
    })?
}
unsafe fn format_cb_session_group_attached_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let s = ft.session()?;
        s.with_group(|group| {
            let mut buffer = ByteBuffer::new();
            with_clients(|clients| {
                for owner in clients {
                    let Some(attached) = owner.attached_session() else {
                        continue;
                    };
                    let attached = attached.downgrade();
                    if group.sessions.iter().any(|member| member.ptr_eq(&attached)) {
                        if !buffer.is_empty() {
                            buffer.append(b",");
                        }
                        format_buf(&mut buffer, c"%s", fmt_args![owner.name()]);
                    }
                }
            });
            format_list_value(&mut buffer)
        })?
    }
}
unsafe fn format_cb_pane_in_mode(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let n: u_int = wp.modes().len() as u_int;
        let value = xasprintf(c"%u", fmt_args![n]);
        Some(value)
    }
}
unsafe fn format_cb_pane_at_top(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;

        let window = pane.window()?;
        let w = window.as_window();
        let status: core::ffi::c_int =
            (w.options_ref()).number(c"pane-border-status") as core::ffi::c_int;
        let flag: core::ffi::c_int = if status == PANE_STATUS_TOP {
            (wp.geometry().yoff == 1 as core::ffi::c_int) as core::ffi::c_int
        } else {
            (wp.geometry().yoff == 0 as core::ffi::c_int) as core::ffi::c_int
        };
        let value = xasprintf(c"%d", fmt_args![flag]);
        Some(value)
    }
}
unsafe fn format_cb_pane_at_bottom(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;

        let window = pane.window()?;
        let w = window.as_window();
        let status: core::ffi::c_int =
            (w.options_ref()).number(c"pane-border-status") as core::ffi::c_int;
        let flag: core::ffi::c_int = if status == PANE_STATUS_BOTTOM {
            (wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int
                == w.dimensions().size.height as core::ffi::c_int - 1 as core::ffi::c_int)
                as core::ffi::c_int
        } else {
            (wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int
                == w.dimensions().size.height as core::ffi::c_int) as core::ffi::c_int
        };
        let value = xasprintf(c"%d", fmt_args![flag]);
        Some(value)
    }
}
unsafe fn format_cb_cursor_character(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;

        let mut value: Option<CString> = None;
        let (cx, cy) = wp.base().cursor();
        let gc = grid_view_get_cell(RustScreen::grid(wp.base()), cx, cy);
        if !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            value = Some(xasprintf(
                c"%.*s",
                fmt_args![
                    gc.data.size as core::ffi::c_int,
                    &gc.data.data[..gc.data.size as usize]
                ],
            ));
        }
        value
    }
}
unsafe fn format_cb_cursor_colour(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(RustColourEngine.to_string(wp.try_screen_ref()?.cursor_colour()))
    }
}
unsafe fn format_cb_mouse_word(ft: &format_tree) -> Option<CString> {
    unsafe {
        let (_, _, mut pane) = cmd_mouse_pane(&ft.m)?;
        let wp = pane.get_mut()?;
        let (x, y) = cmd_mouse_at(wp, &ft.m, 0)?;
        if !wp.modes().is_empty() {
            if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
                return window_copy_get_word(wp, x, y);
            }
            return None;
        }
        let grid = wp.base().grid();
        format_grid_word(grid, x, grid.hsize.wrapping_add(y))
    }
}
unsafe fn format_cb_mouse_hyperlink(ft: &format_tree) -> Option<CString> {
    unsafe {
        let (_, _, mut pane) = cmd_mouse_pane(&ft.m)?;
        let wp = pane.get_mut()?;
        let (x, y) = cmd_mouse_at(wp, &ft.m, 0)?;
        if !wp.modes().is_empty() {
            if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
                return window_copy_get_hyperlink(wp, x, y);
            }
            return None;
        }
        let grid = wp.base().grid();
        format_grid_hyperlink(grid, x, grid.hsize.wrapping_add(y), &wp.screen_ref())
    }
}
unsafe fn format_cb_mouse_line(ft: &format_tree) -> Option<CString> {
    unsafe {
        let (_, _, mut pane) = cmd_mouse_pane(&ft.m)?;
        let wp = pane.get_mut()?;
        let (_, y) = cmd_mouse_at(wp, &ft.m, 0)?;
        if !wp.modes().is_empty() {
            if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
                return Some(window_copy_get_line(wp, y));
            }
            return None;
        }
        let grid = wp.base().grid();
        Some(format_grid_line(grid, grid.hsize.wrapping_add(y)))
    }
}
unsafe fn format_cb_mouse_status_line(ft: &format_tree) -> Option<CString> {
    if ft.m.valid == 0 {
        return None;
    }
    let _ = ft
        .drawn_client()
        .filter(|c| unsafe { c.as_tty() }.flags & TTY_STARTED != 0)?;
    let y: u_int = if ft.m.statusat == 0 as core::ffi::c_int && ft.m.y < ft.m.statuslines {
        ft.m.y
    } else if ft.m.statusat > 0 as core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
        ft.m.y.wrapping_sub(ft.m.statusat as u_int)
    } else {
        return None;
    };
    let value = xasprintf(c"%u", fmt_args![y]);
    Some(value)
}
unsafe fn format_cb_mouse_status_range(ft: &format_tree) -> Option<CString> {
    unsafe {
        let x: u_int;
        let y: u_int;
        if ft.m.valid == 0 {
            return None;
        }
        let c = ft
            .drawn_client()
            .filter(|c| c.as_tty().flags & TTY_STARTED != 0)?;
        if ft.m.statusat == 0 as core::ffi::c_int && ft.m.y < ft.m.statuslines {
            x = ft.m.x;
            y = ft.m.y;
        } else if ft.m.statusat > 0 as core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
            x = ft.m.x;
            y = ft.m.y.wrapping_sub(ft.m.statusat as u_int);
        } else {
            return None;
        }
        let sr = c.status_range(x, y)?;
        match sr.type_0 {
            STYLE_RANGE_NONE => return None,
            STYLE_RANGE_LEFT => {
                return Some(format_callback_copy(c"left"));
            }
            STYLE_RANGE_RIGHT => {
                return Some(format_callback_copy(c"right"));
            }
            STYLE_RANGE_PANE => {
                return Some(format_callback_copy(c"pane"));
            }
            STYLE_RANGE_WINDOW => {
                return Some(format_callback_copy(c"window"));
            }
            STYLE_RANGE_SESSION => {
                return Some(format_callback_copy(c"session"));
            }
            STYLE_RANGE_USER => {
                let bytes = sr.string.map(|byte| byte as u8);
                return Some(format_callback_copy(
                    CStr::from_bytes_until_nul(&bytes).ok()?,
                ));
            }
            STYLE_RANGE_CONTROL => {
                return Some(format_callback_copy(c"control"));
            }
            _ => {}
        }
        None
    }
}
unsafe fn format_cb_alternate_on(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(if wp.base().is_alternate() {
            c"1"
        } else {
            c"0"
        }))
    }
}
unsafe fn format_cb_alternate_saved_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.base().saved_cursor().0]))
    }
}
unsafe fn format_cb_alternate_saved_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.base().saved_cursor().1]))
    }
}
unsafe fn format_cb_bracket_paste_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.try_screen_ref()?.mode() & MODE_BRACKETPASTE != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
fn format_cb_buffer_name(ft: &format_tree) -> Option<CString> {
    let name = ft.buffer_name()?;
    with_paste_buffers(|buffers| {
        buffers
            .get(name)
            .map(|buffer| format_callback_copy(buffer.name))
    })
}
fn format_cb_buffer_sample(ft: &format_tree) -> Option<CString> {
    let name = ft.buffer_name()?;
    with_paste_buffers(|buffers| buffers.sample(name))
}
fn format_cb_buffer_full(ft: &format_tree) -> Option<CString> {
    let name = ft.buffer_name()?;
    with_paste_buffers(|buffers| {
        buffers.get(name).map(|buffer| {
            let bytes = buffer.data;
            let end = bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(bytes.len());
            CString::new(&bytes[..end]).expect("paste buffer data has no NUL")
        })
    })
}
fn format_cb_buffer_size(ft: &format_tree) -> Option<CString> {
    let name = ft.buffer_name()?;
    with_paste_buffers(|buffers| {
        buffers
            .get(name)
            .map(|buffer| format_printf(c"%zu", fmt_args![buffer.data.len() as size_t]))
    })
}
unsafe fn format_cb_client_cell_height(ft: &format_tree) -> Option<CString> {
    if let Some(c) = ft.drawn_client()
        && unsafe { c.as_tty() }.flags & TTY_STARTED != 0
    {
        return Some(format_printf(
            c"%u",
            fmt_args![unsafe { c.as_tty() }.ypixel],
        ));
    }
    None
}
unsafe fn format_cb_client_cell_width(ft: &format_tree) -> Option<CString> {
    if let Some(c) = ft.drawn_client()
        && unsafe { c.as_tty() }.flags & TTY_STARTED != 0
    {
        return Some(format_printf(
            c"%u",
            fmt_args![unsafe { c.as_tty() }.xpixel],
        ));
    }
    None
}
unsafe fn format_cb_client_control_mode(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            if unsafe { c.flags() } & CLIENT_CONTROL as uint64_t != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
fn format_cb_client_discarded(ft: &format_tree) -> Option<CString> {
    if let Some(c) = ft.drawn_client() {
        return Some(format_printf(c"%zu", fmt_args![c.discarded()]));
    }
    None
}
unsafe fn format_cb_client_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        if let Some(mut c) = ft.drawn_client() {
            return Some(c.flag_names());
        }
        None
    }
}
unsafe fn format_cb_client_height(ft: &format_tree) -> Option<CString> {
    if let Some(c) = ft.drawn_client()
        && unsafe { c.as_tty() }.flags & TTY_STARTED != 0
    {
        return Some(format_printf(c"%u", fmt_args![unsafe { c.as_tty() }.sy]));
    }
    None
}
fn format_cb_client_key_table(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            return Some(format_callback_copy(c.keytable()?.name().as_c_str()));
        }
        None
    }
}
unsafe fn format_cb_client_last_session(ft: &format_tree) -> Option<CString> {
    unsafe {
        if let Some(c) = ft.drawn_client()
            && let Some(last) = c.last_session()
            && last.is_registered()
        {
            return Some(format_callback_copy(
                last.name().as_deref().expect("the session has a name"),
            ));
        }
        None
    }
}
unsafe fn format_cb_client_name(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            return Some(format_callback_copy(unsafe { c.name() }.unwrap_or(c"")));
        }
        None
    }
}
fn format_cb_client_pid(ft: &format_tree) -> Option<CString> {
    if let Some(c) = ft.drawn_client() {
        return Some(format_printf(
            c"%ld",
            fmt_args![{ c.pid() } as core::ffi::c_long],
        ));
    }
    None
}
unsafe fn format_cb_client_prefix(ft: &format_tree) -> Option<CString> {
    unsafe {
        if let Some(c) = ft.drawn_client() {
            let name = c.default_key_table();
            if c.keytable()?.name().as_c_str() == name.as_ref() {
                return Some(format_callback_copy(c"0"));
            }
            return Some(format_callback_copy(c"1"));
        }
        None
    }
}
unsafe fn format_cb_client_readonly(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            if unsafe { c.flags() } & CLIENT_READONLY as uint64_t != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
fn format_cb_client_session(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client()
            && let Some(session) = c.attached_session()
        {
            return Some(format_callback_copy(
                session.name().as_deref().expect("the session has a name"),
            ));
        }
        None
    }
}
unsafe fn format_cb_client_termfeatures(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            return Some(RustTerminalFeatureSet.names(unsafe { c.terminal_features() }));
        }
        None
    }
}
fn format_cb_client_termname(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            return Some(format_callback_copy(
                { c.terminal_name() }.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
fn format_cb_client_termtype(ft: &format_tree) -> Option<CString> {
    Some(ft.drawn_client()?.terminal_type().unwrap_or_default())
}
unsafe fn format_cb_client_tty(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            return Some(format_callback_copy(
                unsafe { c.ttyname_ref() }.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
fn format_cb_client_uid(ft: &format_tree) -> Option<CString> {
    {
        let uid: uid_t;
        if let Some(c) = ft.drawn_client() {
            uid = (c.peer_handle()).uid();
            if uid != -(1 as core::ffi::c_int) as uid_t {
                return Some(format_printf(c"%ld", fmt_args![uid as core::ffi::c_long]));
            }
        }
        None
    }
}
unsafe fn format_cb_client_user(ft: &format_tree) -> Option<CString> {
    unsafe { ft.drawn_client()?.user_name() }
}
unsafe fn format_cb_client_utf8(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            if unsafe { c.flags() } & CLIENT_UTF8 as uint64_t != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_client_width(ft: &format_tree) -> Option<CString> {
    if let Some(c) = ft.drawn_client() {
        return Some(format_printf(c"%u", fmt_args![unsafe { c.as_tty() }.sx]));
    }
    None
}
fn format_cb_client_written(ft: &format_tree) -> Option<CString> {
    if let Some(c) = ft.drawn_client() {
        return Some(format_printf(c"%zu", fmt_args![c.written()]));
    }
    None
}
fn format_cb_client_theme(ft: &format_tree) -> Option<CString> {
    {
        if let Some(c) = ft.drawn_client() {
            match c.theme() {
                THEME_DARK => {
                    return Some(format_callback_copy(c"dark"));
                }
                THEME_LIGHT => {
                    return Some(format_callback_copy(c"light"));
                }
                THEME_UNKNOWN => return None,
                _ => {}
            }
        }
        None
    }
}
fn format_cb_config_files(_ft: &format_tree) -> Option<CString> {
    {
        let mut bytes: Vec<u8> = Vec::new();
        for file in &configuration_files() {
            bytes.extend_from_slice(file.as_bytes());
            bytes.push(b',');
        }
        bytes.pop();
        Some(CString::new(bytes).expect("a config file path holds no nul"))
    }
}
unsafe fn format_cb_cursor_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_CURSOR != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_cursor_shape(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            match wp.try_screen_ref()?.cursor_style() {
                SCREEN_CURSOR_BLOCK => c"block",
                SCREEN_CURSOR_UNDERLINE => c"underline",
                SCREEN_CURSOR_BAR => c"bar",
                _ => c"default",
            },
        ))
    }
}
unsafe fn format_cb_cursor_very_visible(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.try_screen_ref()?.mode() & MODE_CURSOR_VERY_VISIBLE != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_cursor_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.base().cursor().0]))
    }
}
unsafe fn format_cb_cursor_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.base().cursor().1]))
    }
}
unsafe fn format_cb_cursor_blinking(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.try_screen_ref()?.mode() & MODE_CURSOR_BLINKING != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_history_limit(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.base().history_limit()]))
    }
}
unsafe fn format_cb_history_size(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(
            c"%u",
            fmt_args![RustScreen::grid(wp.base()).hsize],
        ))
    }
}
unsafe fn format_cb_insert_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_INSERT != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_keypad_cursor_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_KCURSOR != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_keypad_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_KKEYPAD != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
fn format_cb_loop_last_flag(ft: &format_tree) -> Option<CString> {
    if ft.flags & FORMAT_LAST != 0 {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_mouse_all_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_MOUSE_ALL != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_mouse_any_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & ALL_MOUSE_MODES != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_mouse_button_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_MOUSE_BUTTON != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_mouse_pane(ft: &format_tree) -> Option<CString> {
    unsafe {
        let (_, _, pane) = cmd_mouse_pane(&ft.m)?;
        Some(format_printf(c"%%%u", fmt_args![pane.id()]))
    }
}
unsafe fn format_cb_mouse_sgr_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_MOUSE_SGR != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_mouse_standard_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_MOUSE_STANDARD != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_mouse_utf8_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_MOUSE_UTF8 != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_mouse_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        if ft.m.valid == 0 {
            return None;
        }
        if let Some((_, _, pane)) = cmd_mouse_pane(&ft.m)
            && let Some(wp) = pane.get()
            && let Some((x, _)) = cmd_mouse_at(wp, &ft.m, 0)
        {
            return Some(format_printf(c"%u", fmt_args![x]));
        }
        if let Some(c) = ft.drawn_client()
            && c.as_tty().flags & TTY_STARTED != 0
        {
            if ft.m.statusat == 0 as core::ffi::c_int && ft.m.y < ft.m.statuslines {
                return Some(format_printf(c"%u", fmt_args![ft.m.x]));
            }
            if ft.m.statusat > 0 as core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
                return Some(format_printf(c"%u", fmt_args![ft.m.x]));
            }
        }
        None
    }
}
unsafe fn format_cb_mouse_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        if ft.m.valid == 0 {
            return None;
        }
        if let Some((_, _, pane)) = cmd_mouse_pane(&ft.m)
            && let Some(wp) = pane.get()
            && let Some((_, y)) = cmd_mouse_at(wp, &ft.m, 0)
        {
            return Some(format_printf(c"%u", fmt_args![y]));
        }
        if let Some(c) = ft.drawn_client()
            && c.as_tty().flags & TTY_STARTED != 0
        {
            if ft.m.statusat == 0 as core::ffi::c_int && ft.m.y < ft.m.statuslines {
                return Some(format_printf(c"%u", fmt_args![ft.m.y]));
            }
            if ft.m.statusat > 0 as core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
                return Some(format_printf(
                    c"%u",
                    fmt_args![ft.m.y.wrapping_sub(ft.m.statusat as u_int)],
                ));
            }
        }
        None
    }
}
fn format_cb_next_session_id(_ft: &format_tree) -> Option<CString> {
    next_session_id().map(|id| format_printf(c"$%u", fmt_args![id]))
}
unsafe fn format_cb_origin_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            if wp.base().mode() & MODE_ORIGIN != 0 {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
unsafe fn format_cb_synchronized_output_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(if wp.base().mode() & MODE_SYNC != 0 {
            c"1"
        } else {
            c"0"
        }))
    }
}
fn format_cb_pane_active(ft: &format_tree) -> Option<CString> {
    let pane = ft.pane_handle()?;
    Some(format_callback_copy(
        if pane.listed_window()?.active_pane_id() == Some(pane.id()) {
            c"1"
        } else {
            c"0"
        },
    ))
}
unsafe fn format_cb_pane_at_left(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if wp.geometry().xoff == 0 as core::ffi::c_int {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_pane_at_right(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int
            == pane.window()?.dimensions().size.width as core::ffi::c_int
        {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_pane_bottom(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(
            c"%d",
            fmt_args![
                wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int - 1 as core::ffi::c_int
            ],
        ))
    }
}
unsafe fn format_cb_pane_dead(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.fd() == -(1 as core::ffi::c_int) && *wp.flags() & PANE_STATUSREADY != 0 {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_pane_dead_signal(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.flags() & PANE_STATUSREADY != 0
            && ((wp.exit_status() & 0x7f as core::ffi::c_int) + 1 as core::ffi::c_int)
                as core::ffi::c_schar as core::ffi::c_int
                >> 1 as core::ffi::c_int
                > 0 as core::ffi::c_int
        {
            let name = sig2name(wp.exit_status() & 0x7f as core::ffi::c_int);
            return Some(format_printf(c"%s", fmt_args![name.as_c_str()]));
        }
        None
    }
}
unsafe fn format_cb_pane_dead_status(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.flags() & PANE_STATUSREADY != 0
            && wp.exit_status() & 0x7f as core::ffi::c_int == 0 as core::ffi::c_int
        {
            return Some(format_printf(
                c"%d",
                fmt_args![(wp.exit_status() & 0xff00 as core::ffi::c_int) >> 8 as core::ffi::c_int],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_dead_time(ft: &format_tree) -> Option<timeval> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.flags() & PANE_STATUSDRAWN != 0 {
            return Some(wp.death_time());
        }
        None
    }
}
fn format_cb_pane_format(ft: &format_tree) -> Option<CString> {
    if ft.type_0 as core::ffi::c_uint == FORMAT_TYPE_PANE as core::ffi::c_int as core::ffi::c_uint {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_pane_height(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.geometry().sy]))
    }
}
unsafe fn format_cb_pane_id(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%%%u", fmt_args![wp.pane_id()]))
    }
}
unsafe fn format_cb_pane_index(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let window = pane.window()?;
        let wp = pane.get()?;
        if let (0, idx) = window_pane_index(&window.as_window(), wp) {
            return Some(format_printf(c"%u", fmt_args![idx]));
        }
        None
    }
}
unsafe fn format_cb_pane_input_off(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.flags() & PANE_INPUTOFF != 0 {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_pane_unseen_changes(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.flags() & PANE_UNSEENCHANGES != 0 {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_pane_key_mode(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(
            match wp.try_screen_ref()?.mode() & EXTENDED_KEY_MODES {
                MODE_KEYS_EXTENDED => c"Ext 1",
                MODE_KEYS_EXTENDED_2 => c"Ext 2",
                _ => c"VT10x",
            },
        ))
    }
}
fn format_cb_pane_last(ft: &format_tree) -> Option<CString> {
    let pane = ft.pane_handle()?;
    Some(format_callback_copy(
        if pane.listed_window()?.as_window().last_panes.first() == Some(&pane) {
            c"1"
        } else {
            c"0"
        },
    ))
}
unsafe fn format_cb_pane_left(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%d", fmt_args![wp.geometry().xoff]))
    }
}
unsafe fn format_cb_pane_marked(ft: &format_tree) -> Option<CString> {
    {
        let pane = ft.pane_handle()?;
        Some(format_callback_copy(
            if server_check_marked() != 0
                && marked_pane
                    .get()
                    .pane_ref()
                    .is_some_and(|marked| marked.id() == pane.id())
            {
                c"1"
            } else {
                c"0"
            },
        ))
    }
}
fn format_cb_pane_marked_set(ft: &format_tree) -> Option<CString> {
    let _pane = ft.pane_handle()?;
    Some(format_callback_copy(if server_check_marked() != 0 {
        c"1"
    } else {
        c"0"
    }))
}
unsafe fn format_cb_pane_mode(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let wme = wp.modes().first()?;
        Some(format_callback_copy(wme.mode().name()))
    }
}
unsafe fn format_cb_pane_path(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(wp.base().path().unwrap_or(c"")))
    }
}
unsafe fn format_cb_pane_pid(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(
            c"%ld",
            fmt_args![*wp.pid() as core::ffi::c_long],
        ))
    }
}
unsafe fn format_cb_pane_pipe(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.pipe_fd() != -(1 as core::ffi::c_int) {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_pane_pipe_pid(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.pipe_fd() != -(1 as core::ffi::c_int) {
            return Some(xasprintf(
                c"%ld",
                fmt_args![*wp.pipe_pid() as core::ffi::c_long],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_pb_progress(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(xasprintf(
            c"%d",
            fmt_args![wp.base().progress_bar().progress],
        ))
    }
}
unsafe fn format_cb_pane_pb_state(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        match wp.base().progress_bar().state {
            PROGRESS_BAR_HIDDEN => {
                return Some(format_callback_copy(c"hidden"));
            }
            PROGRESS_BAR_NORMAL => {
                return Some(format_callback_copy(c"normal"));
            }
            PROGRESS_BAR_ERROR => {
                return Some(format_callback_copy(c"error"));
            }
            PROGRESS_BAR_INDETERMINATE => {
                return Some(format_callback_copy(c"indeterminate"));
            }
            PROGRESS_BAR_PAUSED => {
                return Some(format_callback_copy(c"paused"));
            }
            _ => {}
        }
        None
    }
}
unsafe fn format_cb_pane_right(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(
            c"%d",
            fmt_args![
                wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int - 1 as core::ffi::c_int
            ],
        ))
    }
}
unsafe fn format_cb_pane_search_string(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let query = wp.query();
        if query.is_none() {
            return Some(format_callback_copy(c""));
        }
        Some(format_callback_copy(query.unwrap_or(c"")))
    }
}
unsafe fn format_cb_pane_synchronized(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if (wp.options_ref()).number(c"synchronize-panes") != 0 {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_pane_title(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(wp.base().title().unwrap_or(c"")))
    }
}
unsafe fn format_cb_pane_top(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%d", fmt_args![wp.geometry().yoff]))
    }
}
unsafe fn format_cb_pane_tty(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(wp.terminal_name()))
    }
}
unsafe fn format_cb_pane_width(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.geometry().sx]))
    }
}
unsafe fn format_cb_pane_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%d", fmt_args![wp.geometry().xoff]))
    }
}
unsafe fn format_cb_pane_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%d", fmt_args![wp.geometry().yoff]))
    }
}
fn format_cb_pane_z(ft: &format_tree) -> Option<CString> {
    {
        let pane = ft.pane_handle()?;
        if let (0, idx) = window_pane_zindex(&pane) {
            return Some(format_printf(c"%u", fmt_args![idx]));
        }
        None
    }
}
unsafe fn format_cb_pane_zoomed_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        if *wp.flags() & PANE_ZOOMED != 0 {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_scroll_region_lower(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.base().region().1]))
    }
}
unsafe fn format_cb_scroll_region_upper(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_printf(c"%u", fmt_args![wp.base().region().0]))
    }
}
fn format_cb_server_sessions(_ft: &format_tree) -> Option<CString> {
    let SESSIONS = SESSIONS_FIELD.get();

    let n = SESSIONS.read().len() as u_int;
    Some(format_printf(c"%u", fmt_args![n]))
}
fn format_cb_session_active(ft: &format_tree) -> Option<CString> {
    {
        let c = ft.drawn_client()?;
        let s = ft.session()?;
        if { c.attached_session() }.is_some_and(|attached| s.ptr_eq(&attached)) {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_session_activity_flag(ft: &format_tree) -> Option<CString> {
    let session = ft.session()?;
    let link = ft.winlink()?;
    let link = link.get()?;
    if unsafe { session.as_session() }.windows.is_empty() {
        return None;
    }
    Some(format_callback_copy(
        if link.flags & WINLINK_ACTIVITY != 0 {
            c"1"
        } else {
            c"0"
        },
    ))
}
unsafe fn format_cb_session_bell_flag(ft: &format_tree) -> Option<CString> {
    let session = ft.session()?;
    let link = unsafe { session.as_session() }.windows.values().next()?;
    Some(format_callback_copy(if link.flags & WINLINK_BELL != 0 {
        c"1"
    } else {
        c"0"
    }))
}
unsafe fn format_cb_session_silence_flag(ft: &format_tree) -> Option<CString> {
    let session = ft.session()?;
    let link = ft.winlink()?;
    let link = link.get()?;
    if unsafe { session.as_session() }.windows.is_empty() {
        return None;
    }
    Some(format_callback_copy(if link.flags & WINLINK_SILENCE != 0 {
        c"1"
    } else {
        c"0"
    }))
}
fn format_cb_session_attached(ft: &format_tree) -> Option<CString> {
    if let Some(s) = (*ft).session() {
        return Some(format_printf(c"%u", fmt_args![s.attached()]));
    }
    None
}
fn format_cb_session_format(ft: &format_tree) -> Option<CString> {
    if ft.type_0 as core::ffi::c_uint
        == FORMAT_TYPE_SESSION as core::ffi::c_int as core::ffi::c_uint
    {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
fn format_cb_session_group(ft: &format_tree) -> Option<CString> {
    let s = ft.session()?;
    s.with_group(|group| session_group_name(group).to_owned())
}
fn format_cb_session_group_attached(ft: &format_tree) -> Option<CString> {
    let s = ft.session()?;
    let count = s.with_group(session_group_attached_count)?;
    Some(format_printf(c"%u", fmt_args![count]))
}
fn format_cb_session_group_many_attached(ft: &format_tree) -> Option<CString> {
    let s = ft.session()?;
    let count = s.with_group(session_group_attached_count)?;
    Some(format_callback_copy(if count > 1 { c"1" } else { c"0" }))
}
fn format_cb_session_group_size(ft: &format_tree) -> Option<CString> {
    let s = ft.session()?;
    let count = s.with_group(session_group_count)?;
    Some(format_printf(c"%u", fmt_args![count]))
}
fn format_cb_session_grouped(ft: &format_tree) -> Option<CString> {
    let s = ft.session()?;
    let grouped = s.with_group(|_| ()).is_some();
    Some(format_callback_copy(if grouped { c"1" } else { c"0" }))
}
fn format_cb_session_id(ft: &format_tree) -> Option<CString> {
    if let Some(s) = (*ft).session() {
        return Some(format_printf(c"$%u", fmt_args![s.id()]));
    }
    None
}
fn format_cb_session_many_attached(ft: &format_tree) -> Option<CString> {
    if let Some(s) = (*ft).session() {
        if s.attached() > 1 as u_int {
            return Some(format_callback_copy(c"1"));
        }
        return Some(format_callback_copy(c"0"));
    }
    None
}
unsafe fn format_cb_session_marked(ft: &format_tree) -> Option<CString> {
    {
        if let Some(s) = (*ft).session() {
            if server_check_marked() != 0
                && marked_pane
                    .get()
                    .session()
                    .is_some_and(|marked| marked.ptr_eq(&s))
            {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
fn format_cb_session_name(ft: &format_tree) -> Option<CString> {
    if let Some(s) = (*ft).session() {
        return Some(format_callback_copy(
            s.name().as_deref().expect("the session has a name"),
        ));
    }
    None
}
fn format_cb_session_path(ft: &format_tree) -> Option<CString> {
    if let Some(s) = (*ft).session() {
        return s.cwd().as_deref().map(format_callback_copy);
    }
    None
}
unsafe fn format_cb_session_windows(ft: &format_tree) -> Option<CString> {
    unsafe {
        if let Some(s) = (*ft).session() {
            return Some(format_printf(
                c"%u",
                fmt_args![winlink_count(&s.as_session().windows)],
            ));
        }
        None
    }
}
unsafe fn format_cb_socket_path(_ft: &format_tree) -> Option<CString> {
    socket_path.get().as_deref().map(format_callback_copy)
}
fn format_cb_version(_ft: &format_tree) -> Option<CString> {
    Some(format_callback_copy(getversion()))
}
fn format_cb_sixel_support(_ft: &format_tree) -> Option<CString> {
    Some(format_callback_copy(c"0"))
}
fn format_cb_active_window_index(ft: &format_tree) -> Option<CString> {
    let session = ft.session()?;
    let link = session.curw()?;
    Some(format_printf(c"%u", fmt_args![link.index()]))
}
unsafe fn format_cb_last_window_index(ft: &format_tree) -> Option<CString> {
    let session = ft.session()?;
    let (_, link) = unsafe { session.as_session() }.windows.last_key_value()?;
    Some(format_printf(c"%u", fmt_args![link.idx]))
}
fn format_cb_window_active(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    let session = link.session()?;
    let active = session
        .curw()
        .is_some_and(|current| current.index() == link.idx);
    Some(format_callback_copy(if active { c"1" } else { c"0" }))
}
fn format_cb_window_activity_flag(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    if link.flags & WINLINK_ACTIVITY != 0 {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
fn format_cb_window_bell_flag(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    if link.flags & WINLINK_BELL != 0 {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_window_bigger(ft: &format_tree) -> Option<CString> {
    unsafe {
        if let Some(c) = ft.drawn_client() {
            let (window_bigger, ..) = tty_window_offset(c.as_tty());
            if window_bigger != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
fn format_cb_window_cell_height(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        return Some(format_printf(
            c"%u",
            fmt_args![w.dimensions().pixels.height],
        ));
    }
    None
}
fn format_cb_window_cell_width(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        return Some(format_printf(c"%u", fmt_args![w.dimensions().pixels.width]));
    }
    None
}
unsafe fn format_cb_window_end_flag(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    let session = link.session()?;
    let last = unsafe { session.as_session() }
        .windows
        .last_key_value()
        .is_some_and(|(_, last)| core::ptr::eq(&**last, &*link));
    Some(format_callback_copy(if last { c"1" } else { c"0" }))
}
unsafe fn format_cb_window_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        let link = (*ft).winlink()?;
        let link = link.get()?;
        Some(window_printable_flags(link, 1 as core::ffi::c_int))
    }
}
fn format_cb_window_format(ft: &format_tree) -> Option<CString> {
    if ft.type_0 as core::ffi::c_uint == FORMAT_TYPE_WINDOW as core::ffi::c_int as core::ffi::c_uint
    {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
fn format_cb_window_height(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        return Some(format_printf(c"%u", fmt_args![w.dimensions().size.height]));
    }
    None
}
fn format_cb_window_id(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        return Some(format_printf(c"@%u", fmt_args![w.window_id()]));
    }
    None
}
fn format_cb_window_index(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    Some(format_printf(c"%d", fmt_args![link.idx]))
}
unsafe fn format_cb_window_last_flag(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    let session = link.session()?;
    let last = unsafe { session.as_session() }.lastw.first() == Some(&link.idx);
    Some(format_callback_copy(if last { c"1" } else { c"0" }))
}
unsafe fn format_cb_window_linked(ft: &format_tree) -> Option<CString> {
    let SESSIONS = SESSIONS_FIELD.get();

    unsafe {
        let link = ft.winlink()?;
        let window = link.get()?.window_handle()?.clone();
        let mut found = false;
        for owner in SESSIONS.read().values() {
            for link in owner.as_session().windows.values() {
                if link
                    .window_handle()
                    .is_some_and(|held| held.ptr_eq(&window))
                {
                    if found {
                        return Some(format_callback_copy(c"1"));
                    }
                    found = true;
                }
            }
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_window_linked_sessions(ft: &format_tree) -> Option<CString> {
    let SESSIONS = SESSIONS_FIELD.get();

    unsafe {
        let link = ft.winlink()?;
        let window = link.get()?.window_handle()?.clone();
        let linked = |s: &session| {
            s.windows.values().any(|link| {
                link.window_handle()
                    .is_some_and(|held| held.ptr_eq(&window))
            })
        };
        let mut count: u_int = 0;
        SESSION_GROUPS.with_borrow(|groups| {
            for group in groups.values() {
                if group
                    .sessions
                    .iter()
                    .find_map(SessionWeak::upgrade)
                    .is_some_and(|member| linked(member.as_session()))
                {
                    count = count.wrapping_add(1);
                }
            }
        });
        for owner in SESSIONS.read().values() {
            let s = owner.as_session();
            if crate::session::session_ref_of(s)
                .and_then(|session| session.with_group(|_| ()))
                .is_none()
                && linked(s)
            {
                count = count.wrapping_add(1);
            }
        }
        Some(format_printf(c"%u", fmt_args![count]))
    }
}
unsafe fn format_cb_window_marked_flag(ft: &format_tree) -> Option<CString> {
    {
        let link = ft.winlink()?;
        if server_check_marked() != 0
            && marked_pane.get().winlink_ref().is_some_and(|marked| {
                marked.index() == link.index() && marked.session().ptr_eq(link.session())
            })
        {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
fn format_cb_window_name(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        return Some(format_printf(c"%s", fmt_args![w.window_name().as_deref()]));
    }
    None
}
unsafe fn format_cb_window_offset_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        if let Some(c) = ft.drawn_client() {
            let (window_bigger, ox, ..) = tty_window_offset(c.as_tty());
            if window_bigger != 0 {
                return Some(format_printf(c"%u", fmt_args![ox]));
            }
            return None;
        }
        None
    }
}
unsafe fn format_cb_window_offset_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        if let Some(c) = ft.drawn_client() {
            let (window_bigger, _ox, oy, ..) = tty_window_offset(c.as_tty());
            if window_bigger != 0 {
                return Some(format_printf(c"%u", fmt_args![oy]));
            }
            return None;
        }
        None
    }
}
fn format_cb_window_panes(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        return Some(format_printf(
            c"%u",
            fmt_args![window_count_panes(&w.as_window(), 1 as core::ffi::c_int)],
        ));
    }
    None
}
unsafe fn format_cb_window_raw_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        let link = (*ft).winlink()?;
        let link = link.get()?;
        Some(window_printable_flags(link, 0 as core::ffi::c_int))
    }
}
fn format_cb_window_silence_flag(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    if link.flags & WINLINK_SILENCE != 0 {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_window_start_flag(ft: &format_tree) -> Option<CString> {
    let link = ft.winlink()?;
    let link = link.get()?;
    let session = link.session()?;
    let first = unsafe { session.as_session() }
        .windows
        .first_key_value()
        .is_some_and(|(_, first)| core::ptr::eq(&**first, &*link));
    Some(format_callback_copy(if first { c"1" } else { c"0" }))
}
fn format_cb_window_width(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        return Some(format_printf(c"%u", fmt_args![w.dimensions().size.width]));
    }
    None
}
fn format_cb_window_zoomed_flag(ft: &format_tree) -> Option<CString> {
    if let Some(w) = (*ft).window() {
        if { w.as_window() }.flags & WINDOW_ZOOMED != 0 {
            return Some(format_callback_copy(c"1"));
        }
        return Some(format_callback_copy(c"0"));
    }
    None
}
unsafe fn format_cb_wrap_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        Some(format_callback_copy(if wp.base().mode() & MODE_WRAP != 0 {
            c"1"
        } else {
            c"0"
        }))
    }
}
fn format_cb_buffer_created(ft: &format_tree) -> Option<timeval> {
    let name = ft.buffer_name()?;
    with_paste_buffers(|buffers| {
        buffers.get(name).map(|buffer| timeval {
            tv_sec: buffer.created as __time_t,
            tv_usec: 0 as __suseconds_t,
        })
    })
}
fn format_cb_client_activity(ft: &format_tree) -> Option<timeval> {
    {
        if let Some(c) = ft.drawn_client() {
            return Some(c.activity_time());
        }
        None
    }
}
fn format_cb_client_created(ft: &format_tree) -> Option<timeval> {
    {
        if let Some(c) = ft.drawn_client() {
            return Some(c.creation_time());
        }
        None
    }
}
fn format_cb_session_activity(ft: &format_tree) -> Option<timeval> {
    if let Some(s) = (*ft).session() {
        return Some(s.activity_time());
    }
    None
}
unsafe fn format_cb_session_created(ft: &format_tree) -> Option<timeval> {
    {
        if let Some(s) = (*ft).session() {
            return Some(unsafe { s.as_session() }.creation_time);
        }
        None
    }
}
unsafe fn format_cb_session_last_attached(ft: &format_tree) -> Option<timeval> {
    {
        if let Some(s) = (*ft).session() {
            return Some(unsafe { s.as_session() }.last_attached_time);
        }
        None
    }
}
unsafe fn format_cb_start_time(_ft: &format_tree) -> Option<timeval> {
    Some(start_time.get())
}
fn format_cb_window_activity(ft: &format_tree) -> Option<timeval> {
    if let Some(w) = (*ft).window() {
        return Some(w.timestamps().activity_time);
    }
    None
}
fn format_cb_buffer_mode_format(_ft: &format_tree) -> Option<CString> {
    WindowMode::Buffer
        .default_format()
        .map(format_callback_copy)
}
fn format_cb_client_mode_format(_ft: &format_tree) -> Option<CString> {
    WindowMode::Client
        .default_format()
        .map(format_callback_copy)
}
fn format_cb_tree_mode_format(_ft: &format_tree) -> Option<CString> {
    WindowMode::Tree.default_format().map(format_callback_copy)
}
fn format_cb_uid(_ft: &format_tree) -> Option<CString> {
    Some(format_printf(
        c"%ld",
        fmt_args![unsafe { getuid() } as core::ffi::c_long],
    ))
}
fn format_cb_user(_ft: &format_tree) -> Option<CString> {
    crate::server::server_proc.with(|state| {
        if let Some(name) = state.cached_user.get() {
            return Some(name.clone());
        }
        let account = UserAccountRecord::lookup_uid(unsafe { getuid() })?;
        let name = account.account_name()?.to_owned();
        Some(state.cached_user.get_or_init(|| name).clone())
    })
}

static format_table: [format_table_entry; 195] = {
    [
        format_table_entry {
            key: c"active_window_index",
            cb: FormatTableCallback::String(format_cb_active_window_index),
        },
        format_table_entry {
            key: c"alternate_on",
            cb: FormatTableCallback::String(format_cb_alternate_on),
        },
        format_table_entry {
            key: c"alternate_saved_x",
            cb: FormatTableCallback::String(format_cb_alternate_saved_x),
        },
        format_table_entry {
            key: c"alternate_saved_y",
            cb: FormatTableCallback::String(format_cb_alternate_saved_y),
        },
        format_table_entry {
            key: c"bracket_paste_flag",
            cb: FormatTableCallback::String(format_cb_bracket_paste_flag),
        },
        format_table_entry {
            key: c"buffer_created",
            cb: FormatTableCallback::Time(format_cb_buffer_created),
        },
        format_table_entry {
            key: c"buffer_full",
            cb: FormatTableCallback::String(format_cb_buffer_full),
        },
        format_table_entry {
            key: c"buffer_mode_format",
            cb: FormatTableCallback::String(format_cb_buffer_mode_format),
        },
        format_table_entry {
            key: c"buffer_name",
            cb: FormatTableCallback::String(format_cb_buffer_name),
        },
        format_table_entry {
            key: c"buffer_sample",
            cb: FormatTableCallback::String(format_cb_buffer_sample),
        },
        format_table_entry {
            key: c"buffer_size",
            cb: FormatTableCallback::String(format_cb_buffer_size),
        },
        format_table_entry {
            key: c"client_activity",
            cb: FormatTableCallback::Time(format_cb_client_activity),
        },
        format_table_entry {
            key: c"client_cell_height",
            cb: FormatTableCallback::String(format_cb_client_cell_height),
        },
        format_table_entry {
            key: c"client_cell_width",
            cb: FormatTableCallback::String(format_cb_client_cell_width),
        },
        format_table_entry {
            key: c"client_control_mode",
            cb: FormatTableCallback::String(format_cb_client_control_mode),
        },
        format_table_entry {
            key: c"client_created",
            cb: FormatTableCallback::Time(format_cb_client_created),
        },
        format_table_entry {
            key: c"client_discarded",
            cb: FormatTableCallback::String(format_cb_client_discarded),
        },
        format_table_entry {
            key: c"client_flags",
            cb: FormatTableCallback::String(format_cb_client_flags),
        },
        format_table_entry {
            key: c"client_height",
            cb: FormatTableCallback::String(format_cb_client_height),
        },
        format_table_entry {
            key: c"client_key_table",
            cb: FormatTableCallback::String(format_cb_client_key_table),
        },
        format_table_entry {
            key: c"client_last_session",
            cb: FormatTableCallback::String(format_cb_client_last_session),
        },
        format_table_entry {
            key: c"client_mode_format",
            cb: FormatTableCallback::String(format_cb_client_mode_format),
        },
        format_table_entry {
            key: c"client_name",
            cb: FormatTableCallback::String(format_cb_client_name),
        },
        format_table_entry {
            key: c"client_pid",
            cb: FormatTableCallback::String(format_cb_client_pid),
        },
        format_table_entry {
            key: c"client_prefix",
            cb: FormatTableCallback::String(format_cb_client_prefix),
        },
        format_table_entry {
            key: c"client_readonly",
            cb: FormatTableCallback::String(format_cb_client_readonly),
        },
        format_table_entry {
            key: c"client_session",
            cb: FormatTableCallback::String(format_cb_client_session),
        },
        format_table_entry {
            key: c"client_termfeatures",
            cb: FormatTableCallback::String(format_cb_client_termfeatures),
        },
        format_table_entry {
            key: c"client_termname",
            cb: FormatTableCallback::String(format_cb_client_termname),
        },
        format_table_entry {
            key: c"client_termtype",
            cb: FormatTableCallback::String(format_cb_client_termtype),
        },
        format_table_entry {
            key: c"client_theme",
            cb: FormatTableCallback::String(format_cb_client_theme),
        },
        format_table_entry {
            key: c"client_tty",
            cb: FormatTableCallback::String(format_cb_client_tty),
        },
        format_table_entry {
            key: c"client_uid",
            cb: FormatTableCallback::String(format_cb_client_uid),
        },
        format_table_entry {
            key: c"client_user",
            cb: FormatTableCallback::String(format_cb_client_user),
        },
        format_table_entry {
            key: c"client_utf8",
            cb: FormatTableCallback::String(format_cb_client_utf8),
        },
        format_table_entry {
            key: c"client_width",
            cb: FormatTableCallback::String(format_cb_client_width),
        },
        format_table_entry {
            key: c"client_written",
            cb: FormatTableCallback::String(format_cb_client_written),
        },
        format_table_entry {
            key: c"config_files",
            cb: FormatTableCallback::String(format_cb_config_files),
        },
        format_table_entry {
            key: c"cursor_blinking",
            cb: FormatTableCallback::String(format_cb_cursor_blinking),
        },
        format_table_entry {
            key: c"cursor_character",
            cb: FormatTableCallback::String(format_cb_cursor_character),
        },
        format_table_entry {
            key: c"cursor_colour",
            cb: FormatTableCallback::String(format_cb_cursor_colour),
        },
        format_table_entry {
            key: c"cursor_flag",
            cb: FormatTableCallback::String(format_cb_cursor_flag),
        },
        format_table_entry {
            key: c"cursor_shape",
            cb: FormatTableCallback::String(format_cb_cursor_shape),
        },
        format_table_entry {
            key: c"cursor_very_visible",
            cb: FormatTableCallback::String(format_cb_cursor_very_visible),
        },
        format_table_entry {
            key: c"cursor_x",
            cb: FormatTableCallback::String(format_cb_cursor_x),
        },
        format_table_entry {
            key: c"cursor_y",
            cb: FormatTableCallback::String(format_cb_cursor_y),
        },
        format_table_entry {
            key: c"history_all_bytes",
            cb: FormatTableCallback::String(format_cb_history_all_bytes),
        },
        format_table_entry {
            key: c"history_bytes",
            cb: FormatTableCallback::String(format_cb_history_bytes),
        },
        format_table_entry {
            key: c"history_limit",
            cb: FormatTableCallback::String(format_cb_history_limit),
        },
        format_table_entry {
            key: c"history_size",
            cb: FormatTableCallback::String(format_cb_history_size),
        },
        format_table_entry {
            key: c"host",
            cb: FormatTableCallback::String(format_cb_host),
        },
        format_table_entry {
            key: c"host_short",
            cb: FormatTableCallback::String(format_cb_host_short),
        },
        format_table_entry {
            key: c"insert_flag",
            cb: FormatTableCallback::String(format_cb_insert_flag),
        },
        format_table_entry {
            key: c"keypad_cursor_flag",
            cb: FormatTableCallback::String(format_cb_keypad_cursor_flag),
        },
        format_table_entry {
            key: c"keypad_flag",
            cb: FormatTableCallback::String(format_cb_keypad_flag),
        },
        format_table_entry {
            key: c"last_window_index",
            cb: FormatTableCallback::String(format_cb_last_window_index),
        },
        format_table_entry {
            key: c"loop_last_flag",
            cb: FormatTableCallback::String(format_cb_loop_last_flag),
        },
        format_table_entry {
            key: c"mouse_all_flag",
            cb: FormatTableCallback::String(format_cb_mouse_all_flag),
        },
        format_table_entry {
            key: c"mouse_any_flag",
            cb: FormatTableCallback::String(format_cb_mouse_any_flag),
        },
        format_table_entry {
            key: c"mouse_button_flag",
            cb: FormatTableCallback::String(format_cb_mouse_button_flag),
        },
        format_table_entry {
            key: c"mouse_hyperlink",
            cb: FormatTableCallback::String(format_cb_mouse_hyperlink),
        },
        format_table_entry {
            key: c"mouse_line",
            cb: FormatTableCallback::String(format_cb_mouse_line),
        },
        format_table_entry {
            key: c"mouse_pane",
            cb: FormatTableCallback::String(format_cb_mouse_pane),
        },
        format_table_entry {
            key: c"mouse_sgr_flag",
            cb: FormatTableCallback::String(format_cb_mouse_sgr_flag),
        },
        format_table_entry {
            key: c"mouse_standard_flag",
            cb: FormatTableCallback::String(format_cb_mouse_standard_flag),
        },
        format_table_entry {
            key: c"mouse_status_line",
            cb: FormatTableCallback::String(format_cb_mouse_status_line),
        },
        format_table_entry {
            key: c"mouse_status_range",
            cb: FormatTableCallback::String(format_cb_mouse_status_range),
        },
        format_table_entry {
            key: c"mouse_utf8_flag",
            cb: FormatTableCallback::String(format_cb_mouse_utf8_flag),
        },
        format_table_entry {
            key: c"mouse_word",
            cb: FormatTableCallback::String(format_cb_mouse_word),
        },
        format_table_entry {
            key: c"mouse_x",
            cb: FormatTableCallback::String(format_cb_mouse_x),
        },
        format_table_entry {
            key: c"mouse_y",
            cb: FormatTableCallback::String(format_cb_mouse_y),
        },
        format_table_entry {
            key: c"next_session_id",
            cb: FormatTableCallback::String(format_cb_next_session_id),
        },
        format_table_entry {
            key: c"origin_flag",
            cb: FormatTableCallback::String(format_cb_origin_flag),
        },
        format_table_entry {
            key: c"pane_active",
            cb: FormatTableCallback::String(format_cb_pane_active),
        },
        format_table_entry {
            key: c"pane_at_bottom",
            cb: FormatTableCallback::String(format_cb_pane_at_bottom),
        },
        format_table_entry {
            key: c"pane_at_left",
            cb: FormatTableCallback::String(format_cb_pane_at_left),
        },
        format_table_entry {
            key: c"pane_at_right",
            cb: FormatTableCallback::String(format_cb_pane_at_right),
        },
        format_table_entry {
            key: c"pane_at_top",
            cb: FormatTableCallback::String(format_cb_pane_at_top),
        },
        format_table_entry {
            key: c"pane_bg",
            cb: FormatTableCallback::String(format_cb_pane_bg),
        },
        format_table_entry {
            key: c"pane_bottom",
            cb: FormatTableCallback::String(format_cb_pane_bottom),
        },
        format_table_entry {
            key: c"pane_current_command",
            cb: FormatTableCallback::String(format_cb_current_command),
        },
        format_table_entry {
            key: c"pane_current_path",
            cb: FormatTableCallback::String(format_cb_current_path),
        },
        format_table_entry {
            key: c"pane_dead",
            cb: FormatTableCallback::String(format_cb_pane_dead),
        },
        format_table_entry {
            key: c"pane_dead_signal",
            cb: FormatTableCallback::String(format_cb_pane_dead_signal),
        },
        format_table_entry {
            key: c"pane_dead_status",
            cb: FormatTableCallback::String(format_cb_pane_dead_status),
        },
        format_table_entry {
            key: c"pane_dead_time",
            cb: FormatTableCallback::Time(format_cb_pane_dead_time),
        },
        format_table_entry {
            key: c"pane_fg",
            cb: FormatTableCallback::String(format_cb_pane_fg),
        },
        format_table_entry {
            key: c"pane_flags",
            cb: FormatTableCallback::String(format_cb_pane_flags),
        },
        format_table_entry {
            key: c"pane_floating_flag",
            cb: FormatTableCallback::String(format_cb_pane_floating_flag),
        },
        format_table_entry {
            key: c"pane_format",
            cb: FormatTableCallback::String(format_cb_pane_format),
        },
        format_table_entry {
            key: c"pane_height",
            cb: FormatTableCallback::String(format_cb_pane_height),
        },
        format_table_entry {
            key: c"pane_id",
            cb: FormatTableCallback::String(format_cb_pane_id),
        },
        format_table_entry {
            key: c"pane_in_mode",
            cb: FormatTableCallback::String(format_cb_pane_in_mode),
        },
        format_table_entry {
            key: c"pane_index",
            cb: FormatTableCallback::String(format_cb_pane_index),
        },
        format_table_entry {
            key: c"pane_input_off",
            cb: FormatTableCallback::String(format_cb_pane_input_off),
        },
        format_table_entry {
            key: c"pane_key_mode",
            cb: FormatTableCallback::String(format_cb_pane_key_mode),
        },
        format_table_entry {
            key: c"pane_last",
            cb: FormatTableCallback::String(format_cb_pane_last),
        },
        format_table_entry {
            key: c"pane_left",
            cb: FormatTableCallback::String(format_cb_pane_left),
        },
        format_table_entry {
            key: c"pane_marked",
            cb: FormatTableCallback::String(format_cb_pane_marked),
        },
        format_table_entry {
            key: c"pane_marked_set",
            cb: FormatTableCallback::String(format_cb_pane_marked_set),
        },
        format_table_entry {
            key: c"pane_mode",
            cb: FormatTableCallback::String(format_cb_pane_mode),
        },
        format_table_entry {
            key: c"pane_path",
            cb: FormatTableCallback::String(format_cb_pane_path),
        },
        format_table_entry {
            key: c"pane_pb_progress",
            cb: FormatTableCallback::String(format_cb_pane_pb_progress),
        },
        format_table_entry {
            key: c"pane_pb_state",
            cb: FormatTableCallback::String(format_cb_pane_pb_state),
        },
        format_table_entry {
            key: c"pane_pid",
            cb: FormatTableCallback::String(format_cb_pane_pid),
        },
        format_table_entry {
            key: c"pane_pipe",
            cb: FormatTableCallback::String(format_cb_pane_pipe),
        },
        format_table_entry {
            key: c"pane_pipe_pid",
            cb: FormatTableCallback::String(format_cb_pane_pipe_pid),
        },
        format_table_entry {
            key: c"pane_right",
            cb: FormatTableCallback::String(format_cb_pane_right),
        },
        format_table_entry {
            key: c"pane_search_string",
            cb: FormatTableCallback::String(format_cb_pane_search_string),
        },
        format_table_entry {
            key: c"pane_start_command",
            cb: FormatTableCallback::String(format_cb_start_command),
        },
        format_table_entry {
            key: c"pane_start_path",
            cb: FormatTableCallback::String(format_cb_start_path),
        },
        format_table_entry {
            key: c"pane_synchronized",
            cb: FormatTableCallback::String(format_cb_pane_synchronized),
        },
        format_table_entry {
            key: c"pane_tabs",
            cb: FormatTableCallback::String(format_cb_pane_tabs),
        },
        format_table_entry {
            key: c"pane_title",
            cb: FormatTableCallback::String(format_cb_pane_title),
        },
        format_table_entry {
            key: c"pane_top",
            cb: FormatTableCallback::String(format_cb_pane_top),
        },
        format_table_entry {
            key: c"pane_tty",
            cb: FormatTableCallback::String(format_cb_pane_tty),
        },
        format_table_entry {
            key: c"pane_unseen_changes",
            cb: FormatTableCallback::String(format_cb_pane_unseen_changes),
        },
        format_table_entry {
            key: c"pane_width",
            cb: FormatTableCallback::String(format_cb_pane_width),
        },
        format_table_entry {
            key: c"pane_x",
            cb: FormatTableCallback::String(format_cb_pane_x),
        },
        format_table_entry {
            key: c"pane_y",
            cb: FormatTableCallback::String(format_cb_pane_y),
        },
        format_table_entry {
            key: c"pane_z",
            cb: FormatTableCallback::String(format_cb_pane_z),
        },
        format_table_entry {
            key: c"pane_zoomed_flag",
            cb: FormatTableCallback::String(format_cb_pane_zoomed_flag),
        },
        format_table_entry {
            key: c"pid",
            cb: FormatTableCallback::String(format_cb_pid),
        },
        format_table_entry {
            key: c"scroll_region_lower",
            cb: FormatTableCallback::String(format_cb_scroll_region_lower),
        },
        format_table_entry {
            key: c"scroll_region_upper",
            cb: FormatTableCallback::String(format_cb_scroll_region_upper),
        },
        format_table_entry {
            key: c"server_sessions",
            cb: FormatTableCallback::String(format_cb_server_sessions),
        },
        format_table_entry {
            key: c"session_active",
            cb: FormatTableCallback::String(format_cb_session_active),
        },
        format_table_entry {
            key: c"session_activity",
            cb: FormatTableCallback::Time(format_cb_session_activity),
        },
        format_table_entry {
            key: c"session_activity_flag",
            cb: FormatTableCallback::String(format_cb_session_activity_flag),
        },
        format_table_entry {
            key: c"session_alert",
            cb: FormatTableCallback::String(format_cb_session_alert),
        },
        format_table_entry {
            key: c"session_alerts",
            cb: FormatTableCallback::String(format_cb_session_alerts),
        },
        format_table_entry {
            key: c"session_attached",
            cb: FormatTableCallback::String(format_cb_session_attached),
        },
        format_table_entry {
            key: c"session_attached_list",
            cb: FormatTableCallback::String(format_cb_session_attached_list),
        },
        format_table_entry {
            key: c"session_bell_flag",
            cb: FormatTableCallback::String(format_cb_session_bell_flag),
        },
        format_table_entry {
            key: c"session_created",
            cb: FormatTableCallback::Time(format_cb_session_created),
        },
        format_table_entry {
            key: c"session_format",
            cb: FormatTableCallback::String(format_cb_session_format),
        },
        format_table_entry {
            key: c"session_group",
            cb: FormatTableCallback::String(format_cb_session_group),
        },
        format_table_entry {
            key: c"session_group_attached",
            cb: FormatTableCallback::String(format_cb_session_group_attached),
        },
        format_table_entry {
            key: c"session_group_attached_list",
            cb: FormatTableCallback::String(format_cb_session_group_attached_list),
        },
        format_table_entry {
            key: c"session_group_list",
            cb: FormatTableCallback::String(format_cb_session_group_list),
        },
        format_table_entry {
            key: c"session_group_many_attached",
            cb: FormatTableCallback::String(format_cb_session_group_many_attached),
        },
        format_table_entry {
            key: c"session_group_size",
            cb: FormatTableCallback::String(format_cb_session_group_size),
        },
        format_table_entry {
            key: c"session_grouped",
            cb: FormatTableCallback::String(format_cb_session_grouped),
        },
        format_table_entry {
            key: c"session_id",
            cb: FormatTableCallback::String(format_cb_session_id),
        },
        format_table_entry {
            key: c"session_last_attached",
            cb: FormatTableCallback::Time(format_cb_session_last_attached),
        },
        format_table_entry {
            key: c"session_many_attached",
            cb: FormatTableCallback::String(format_cb_session_many_attached),
        },
        format_table_entry {
            key: c"session_marked",
            cb: FormatTableCallback::String(format_cb_session_marked),
        },
        format_table_entry {
            key: c"session_name",
            cb: FormatTableCallback::String(format_cb_session_name),
        },
        format_table_entry {
            key: c"session_path",
            cb: FormatTableCallback::String(format_cb_session_path),
        },
        format_table_entry {
            key: c"session_silence_flag",
            cb: FormatTableCallback::String(format_cb_session_silence_flag),
        },
        format_table_entry {
            key: c"session_stack",
            cb: FormatTableCallback::String(format_cb_session_stack),
        },
        format_table_entry {
            key: c"session_windows",
            cb: FormatTableCallback::String(format_cb_session_windows),
        },
        format_table_entry {
            key: c"sixel_support",
            cb: FormatTableCallback::String(format_cb_sixel_support),
        },
        format_table_entry {
            key: c"socket_path",
            cb: FormatTableCallback::String(format_cb_socket_path),
        },
        format_table_entry {
            key: c"start_time",
            cb: FormatTableCallback::Time(format_cb_start_time),
        },
        format_table_entry {
            key: c"synchronized_output_flag",
            cb: FormatTableCallback::String(format_cb_synchronized_output_flag),
        },
        format_table_entry {
            key: c"tree_mode_format",
            cb: FormatTableCallback::String(format_cb_tree_mode_format),
        },
        format_table_entry {
            key: c"uid",
            cb: FormatTableCallback::String(format_cb_uid),
        },
        format_table_entry {
            key: c"user",
            cb: FormatTableCallback::String(format_cb_user),
        },
        format_table_entry {
            key: c"version",
            cb: FormatTableCallback::String(format_cb_version),
        },
        format_table_entry {
            key: c"window_active",
            cb: FormatTableCallback::String(format_cb_window_active),
        },
        format_table_entry {
            key: c"window_active_clients",
            cb: FormatTableCallback::String(format_cb_window_active_clients),
        },
        format_table_entry {
            key: c"window_active_clients_list",
            cb: FormatTableCallback::String(format_cb_window_active_clients_list),
        },
        format_table_entry {
            key: c"window_active_sessions",
            cb: FormatTableCallback::String(format_cb_window_active_sessions),
        },
        format_table_entry {
            key: c"window_active_sessions_list",
            cb: FormatTableCallback::String(format_cb_window_active_sessions_list),
        },
        format_table_entry {
            key: c"window_activity",
            cb: FormatTableCallback::Time(format_cb_window_activity),
        },
        format_table_entry {
            key: c"window_activity_flag",
            cb: FormatTableCallback::String(format_cb_window_activity_flag),
        },
        format_table_entry {
            key: c"window_bell_flag",
            cb: FormatTableCallback::String(format_cb_window_bell_flag),
        },
        format_table_entry {
            key: c"window_bigger",
            cb: FormatTableCallback::String(format_cb_window_bigger),
        },
        format_table_entry {
            key: c"window_cell_height",
            cb: FormatTableCallback::String(format_cb_window_cell_height),
        },
        format_table_entry {
            key: c"window_cell_width",
            cb: FormatTableCallback::String(format_cb_window_cell_width),
        },
        format_table_entry {
            key: c"window_end_flag",
            cb: FormatTableCallback::String(format_cb_window_end_flag),
        },
        format_table_entry {
            key: c"window_flags",
            cb: FormatTableCallback::String(format_cb_window_flags),
        },
        format_table_entry {
            key: c"window_format",
            cb: FormatTableCallback::String(format_cb_window_format),
        },
        format_table_entry {
            key: c"window_height",
            cb: FormatTableCallback::String(format_cb_window_height),
        },
        format_table_entry {
            key: c"window_id",
            cb: FormatTableCallback::String(format_cb_window_id),
        },
        format_table_entry {
            key: c"window_index",
            cb: FormatTableCallback::String(format_cb_window_index),
        },
        format_table_entry {
            key: c"window_last_flag",
            cb: FormatTableCallback::String(format_cb_window_last_flag),
        },
        format_table_entry {
            key: c"window_layout",
            cb: FormatTableCallback::String(format_cb_window_layout),
        },
        format_table_entry {
            key: c"window_linked",
            cb: FormatTableCallback::String(format_cb_window_linked),
        },
        format_table_entry {
            key: c"window_linked_sessions",
            cb: FormatTableCallback::String(format_cb_window_linked_sessions),
        },
        format_table_entry {
            key: c"window_linked_sessions_list",
            cb: FormatTableCallback::String(format_cb_window_linked_sessions_list),
        },
        format_table_entry {
            key: c"window_marked_flag",
            cb: FormatTableCallback::String(format_cb_window_marked_flag),
        },
        format_table_entry {
            key: c"window_name",
            cb: FormatTableCallback::String(format_cb_window_name),
        },
        format_table_entry {
            key: c"window_offset_x",
            cb: FormatTableCallback::String(format_cb_window_offset_x),
        },
        format_table_entry {
            key: c"window_offset_y",
            cb: FormatTableCallback::String(format_cb_window_offset_y),
        },
        format_table_entry {
            key: c"window_panes",
            cb: FormatTableCallback::String(format_cb_window_panes),
        },
        format_table_entry {
            key: c"window_raw_flags",
            cb: FormatTableCallback::String(format_cb_window_raw_flags),
        },
        format_table_entry {
            key: c"window_silence_flag",
            cb: FormatTableCallback::String(format_cb_window_silence_flag),
        },
        format_table_entry {
            key: c"window_stack_index",
            cb: FormatTableCallback::String(format_cb_window_stack_index),
        },
        format_table_entry {
            key: c"window_start_flag",
            cb: FormatTableCallback::String(format_cb_window_start_flag),
        },
        format_table_entry {
            key: c"window_visible_layout",
            cb: FormatTableCallback::String(format_cb_window_visible_layout),
        },
        format_table_entry {
            key: c"window_width",
            cb: FormatTableCallback::String(format_cb_window_width),
        },
        format_table_entry {
            key: c"window_zoomed_flag",
            cb: FormatTableCallback::String(format_cb_window_zoomed_flag),
        },
        format_table_entry {
            key: c"wrap_flag",
            cb: FormatTableCallback::String(format_cb_wrap_flag),
        },
    ]
};
fn format_table_get(key: &CStr) -> Option<&'static format_table_entry> {
    {
        match format_table.binary_search_by(|entry| entry.key.cmp(key)) {
            Ok(index) => Some(&format_table[index]),
            Err(_) => None,
        }
    }
}
pub fn format_merge(ft: &mut format_tree, from: &format_tree) {
    let entries: Vec<(CString, CString)> = from
        .tree
        .iter()
        .filter_map(|(key, fe)| fe.value.clone().map(|value| (key.clone(), value)))
        .collect();
    for (key, value) in entries {
        format_add(ft, &key, c"%s", fmt_args![value.as_c_str()]);
    }
}
fn format_create_add_item(ft: &mut format_tree, item: &cmdq_item) {
    let event_state_ref = item.state_ref();
    let event = event_state_ref.state().event.clone();
    item.merge_formats(ft);
    ft.m = event.m;
}
pub fn format_create(
    c: Option<&client>,
    item: Option<&cmdq_item>,
    tag: core::ffi::c_int,
    flags: core::ffi::c_int,
) -> Box<format_tree> {
    format_create_for_client(c.and_then(client_ref_of).as_ref(), item, tag, flags)
}
/// Creates an empty context retaining the optional client for format jobs.
///
/// Copies queue formats and mouse state from `item`, observing the item weakly.
/// Neither the client nor the item's targets supply entity defaults here;
/// an absent client stays absent. No expansion or callbacks run.
pub(crate) fn format_create_for_client(
    c: Option<&ClientRef>,
    item: Option<&cmdq_item>,
    tag: core::ffi::c_int,
    flags: core::ffi::c_int,
) -> Box<format_tree> {
    let mut ft = Box::new(format_tree {
        client: c.cloned(),
        flags,
        tag: tag as u_int,
        ..Default::default()
    });
    ft.set_item(item);
    if let Some(item) = item {
        format_create_add_item(&mut ft, item);
    }
    ft
}
pub unsafe fn format_log_debug(ft: &mut format_tree, prefix: &CStr) {
    unsafe {
        format_each(ft, |key, value| {
            log_debug(c"%s: %s=%s", fmt_args![prefix, key, value]);
        });
    }
}
/// Enumerates builtin, plugin and context entries through synchronous callbacks.
///
/// Evaluates builtin callbacks and plugin resolvers, and fills deferred custom
/// entries in `ft` before passing their values to `cb`. Key/value borrows supplied
/// to `cb` last only for that invocation. This enumeration does not itself expand
/// the resulting values as format strings.
///
/// # Safety
/// Run on the server thread without conflicting client/session/window/pane or
/// option payload access. Builtin and plugin resolvers run synchronously; callers
/// must not retain subsystem payload borrows across enumeration or its callback.
/// The callback must not reenter this format tree through an alias.
pub unsafe fn format_each(ft: &mut format_tree, mut cb: impl FnMut(&CStr, &CStr)) {
    unsafe {
        for fte in &format_table {
            match fte.cb {
                FormatTableCallback::Time(table_cb) => {
                    if let Some(tv) = table_cb(ft) {
                        let s = xasprintf(c"%lld", fmt_args![tv.tv_sec as core::ffi::c_longlong]);
                        cb(fte.key, &s);
                    }
                }
                FormatTableCallback::String(table_cb) => {
                    if let Some(value) = table_cb(ft) {
                        cb(fte.key, &value);
                    }
                }
            }
        }
        for (key, value) in crate::plugin::each(ft.pane_handle().map(|pane| pane.id())) {
            cb(&key, &value);
        }
        let keys: Vec<CString> = ft.tree.keys().cloned().collect();
        for key in keys {
            let time = ft.tree[&key].time;
            if time != 0 as time_t {
                let s = xasprintf(c"%lld", fmt_args![time as core::ffi::c_longlong]);
                cb(&key, &s);
            } else {
                format_entry_fill(ft, &key);
                let value = ft.tree[&key].value.clone().unwrap_or_default();
                cb(&key, &value);
            }
        }
    }
}
/// Fills in the entry `key` names so that it holds the string the rest of the
/// walk reads. An entry that already holds one, or has no callback, is left
/// as it is. The callback is asked outside the map, since it reads the tree
/// the entry sits in.
unsafe fn format_entry_fill(ft: &mut format_tree, key: &CStr) {
    unsafe {
        let Some(fe) = ft.tree.get(key) else {
            return;
        };
        if fe.value.is_some() {
            return;
        }
        let Some(cb) = fe.cb else {
            return;
        };
        let value = cb(ft).unwrap_or_default();
        if let Some(fe) = ft.tree.get_mut(key) {
            fe.value = Some(value);
        }
    }
}

/// The entry `key` names in `ft`, made and put into the tree if it is not
/// there yet. Whatever value it held is given up, since every caller sets one
/// of its own.
fn format_entry_for<'a>(ft: &'a mut format_tree, key: &CStr) -> &'a mut format_entry {
    {
        let fe = ft.tree.entry(key.to_owned()).or_insert(format_entry {
            value: None,
            time: 0 as time_t,
            cb: None,
        });
        fe.value = None;
        fe
    }
}

pub fn format_add(ft: &mut format_tree, key: &CStr, fmt: &CStr, args: &[FmtArg]) {
    let fe = format_entry_for(ft, key);
    fe.cb = None;
    fe.time = 0 as time_t;
    fe.value = Some(format_alloc(fmt, args));
}
pub fn format_add_tv(ft: &mut format_tree, key: &CStr, tv: &timeval) {
    let fe = format_entry_for(ft, key);
    fe.cb = None;
    fe.time = tv.tv_sec as time_t;
    fe.value = None;
}
pub fn format_add_cb(ft: &mut format_tree, key: &CStr, cb: format_entry_cb) {
    let fe = format_entry_for(ft, key);
    fe.cb = cb;
    fe.time = 0 as time_t;
    fe.value = None;
}
fn format_quote_shell(s: &CStr) -> CString {
    {
        let mut out = Vec::with_capacity(s.to_bytes().len().saturating_mul(2));
        for byte in s.to_bytes() {
            if b"|&;<>()$`\\\"'*?[# =%".contains(byte) {
                out.push(b'\\');
            }
            out.push(*byte);
        }
        CString::new(out).expect("format shell quote output has no NUL")
    }
}
fn format_quote_style(s: &CStr) -> CString {
    {
        let mut out = Vec::with_capacity(s.to_bytes().len().saturating_mul(2));
        for byte in s.to_bytes() {
            if *byte == b'#' {
                out.push(b'#');
            }
            out.push(*byte);
        }
        CString::new(out).expect("format style quote output has no NUL")
    }
}
pub fn format_pretty_time(t: time_t, seconds: core::ffi::c_int) -> CString {
    let now = unsafe { time(core::ptr::null_mut()) }.max(t);
    let age = now - t;
    let now_tm = tm::local(now).unwrap_or_default();
    let tm = tm::local(t).unwrap_or_default();
    let format = if age < 24 * 3600 {
        if seconds != 0 { c"%H:%M:%S" } else { c"%H:%M" }
    } else if tm.tm_year == now_tm.tm_year && tm.tm_mon == now_tm.tm_mon || age < 28 * 24 * 3600 {
        c"%a%d"
    } else if tm.tm_year == now_tm.tm_year && tm.tm_mon < now_tm.tm_mon
        || tm.tm_year == now_tm.tm_year - 1 && tm.tm_mon > now_tm.tm_mon
    {
        c"%d%b"
    } else {
        c"%h%y"
    };
    format_strftime(9, format, &tm).unwrap_or_default()
}
unsafe fn format_find_option(ft: &format_tree, key: &CStr) -> Option<CString> {
    unsafe {
        let mut index = 0;
        let name = RustOptionsEngine.parse(key, &mut index)?;
        let read = |options: &RustOptionsRef| {
            options.with_entry(&name, false, |entry| {
                entry.map(|entry| RustOptionsEngine.display(entry, index, 1))
            })
        };
        if let Some(value) = read(
            global_options
                .get()
                .as_ref()
                .expect("global options are initialized"),
        ) {
            return Some(value);
        }
        if let Some(handle) = ft.pane_handle()
            && let Some(pane) = handle.get()
            && let Some(value) = read(pane.options_ref())
        {
            return Some(value);
        }
        if let Some(window) = ft.window()
            && let Some(value) = read(&window.options())
        {
            return Some(value);
        }
        if let Some(value) = read(
            global_w_options
                .get()
                .as_ref()
                .expect("global options are initialized"),
        ) {
            return Some(value);
        }
        if let Some(session) = ft.session()
            && let Some(value) = read(&session.options())
        {
            return Some(value);
        }
        read(
            global_s_options
                .get()
                .as_ref()
                .expect("global options are initialized"),
        )
    }
}

unsafe fn format_find(
    ft: &mut format_tree,
    key: &CStr,
    modifiers: core::ffi::c_int,
    time_format: Option<&CStr>,
) -> Option<CString> {
    unsafe {
        let current_block: u64;
        let mut found = format_find_option(ft, key);
        let mut s = [0u8; 512];
        let mut t: time_t = 0 as time_t;
        let tm;
        if found.is_none() {
            if let Some(fte) = format_table_get(key) {
                match fte.cb {
                    FormatTableCallback::Time(table_cb) => {
                        if let Some(value) = table_cb(ft) {
                            t = value.tv_sec as time_t;
                        }
                    }
                    FormatTableCallback::String(table_cb) => {
                        if let Some(value) = table_cb(ft) {
                            found = Some(value);
                        }
                    }
                }
            } else if let Some(value) =
                crate::plugin::find(ft.pane_handle().map(|pane| pane.id()), key)
            {
                found = Some(value);
            } else {
                let wanted = key;
                if let Some(time) = ft.tree.get(wanted).map(|fe| fe.time) {
                    if time != 0 as time_t {
                        t = time;
                    } else {
                        format_entry_fill(ft, wanted);
                        found = ft.tree[wanted].value.clone();
                    }
                } else {
                    if !modifiers & FORMAT_TIMESTRING != 0 {
                        let session_value = (*ft).session().and_then(|s| {
                            s.as_session()
                                .environ_ref()
                                .find(key)
                                .map(|entry| entry.value.map(CStr::to_owned))
                        });
                        found = session_value.unwrap_or_else(|| {
                            with_global_environment(|env| {
                                env.find(key)
                                    .and_then(|entry| entry.value.map(CStr::to_owned))
                            })
                        });
                        if found.is_some() {
                            current_block = 9836515120145841630;
                        } else {
                            current_block = 17184638872671510253;
                        }
                    } else {
                        current_block = 17184638872671510253;
                    }
                    match current_block {
                        9836515120145841630 => {}
                        _ => return None,
                    }
                }
            }
        }
        if modifiers & FORMAT_TIMESTRING != 0 {
            if t == 0 as time_t {
                let found_value = found.as_ref()?;
                t = strtonum(
                    found_value,
                    0 as core::ffi::c_longlong,
                    INT64_MAX as core::ffi::c_longlong,
                )
                .map_or(0 as time_t, |value| value as time_t);
                drop(found.take());
            }
            if t == 0 as time_t {
                return None;
            }
            if modifiers & FORMAT_PRETTY != 0 {
                found = Some(format_pretty_time(t, 0 as core::ffi::c_int));
            } else {
                if let Some(time_format) = time_format {
                    tm = crate::types::tm::local(t).unwrap_or_default();
                    found =
                        Some(format_strftime(512 as size_t, time_format, &tm).unwrap_or_default());
                } else {
                    ctime_r(&t, s.as_mut_ptr().cast());
                    let time = CStr::from_bytes_until_nul(&s).ok()?;
                    let bytes = time.to_bytes();
                    let end = bytes
                        .iter()
                        .position(|&byte| byte == b'\n')
                        .unwrap_or(bytes.len());
                    found = Some(CString::new(&bytes[..end]).expect("time string has no NUL"));
                }
            }
            return found;
        }
        if t != 0 as time_t {
            found = Some(xasprintf(c"%lld", fmt_args![t as core::ffi::c_longlong]));
        } else if found.is_none() {
            return None;
        }
        if modifiers & FORMAT_BASENAME != 0 {
            let mut path = found.take().unwrap().into_bytes_with_nul();
            let basename = __xpg_basename(path.as_mut_ptr() as *mut core::ffi::c_char);
            found = Some(CStr::from_ptr(basename).to_owned());
        }
        if modifiers & FORMAT_DIRNAME != 0 {
            let mut path = found.take().unwrap().into_bytes_with_nul();
            let dirname = dirname(path.as_mut_ptr() as *mut core::ffi::c_char);
            found = Some(CStr::from_ptr(dirname).to_owned());
        }
        if modifiers & FORMAT_QUOTE_SHELL != 0 {
            let value = found.take().unwrap();
            found = Some(format_quote_shell(&value));
        }
        if modifiers & FORMAT_QUOTE_STYLE != 0 {
            let value = found.take().unwrap();
            found = Some(format_quote_style(&value));
        }
        if modifiers & FORMAT_QUOTE_ARGUMENTS != 0 {
            let value = found.take().unwrap();
            found = Some(RustArgumentTextCodec.escape(value.as_c_str()));
        }
        found
    }
}
unsafe fn format_check_time(
    ft: &mut format_tree,
    es: &mut format_expand_state,
) -> core::ffi::c_int {
    unsafe {
        let mut t: uint64_t = get_timer();
        if t.wrapping_sub(es.start_time) < FORMAT_TIME_LIMIT as uint64_t {
            return 1 as core::ffi::c_int;
        }
        t = t.wrapping_sub(es.start_time);
        format_log1(
            ft,
            es,
            c"format_check_time",
            c"reached time limit (%llu)",
            fmt_args![t as core::ffi::c_ulonglong],
        );
        0 as core::ffi::c_int
    }
}
unsafe fn format_unescape(ft: &mut format_tree, es: &mut format_expand_state, s: &CStr) -> CString {
    unsafe {
        let mut brackets: core::ffi::c_int = 0 as core::ffi::c_int;
        let bytes = s.to_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if format_check_time(ft, es) == 0 {
                return CString::default();
            }
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            if bytes[i] == b'#' && bytes.get(i + 1) == Some(&b'{') {
                brackets += 1;
            }
            if brackets == 0 && bytes[i] == b'#' && b",#{}:\0".contains(&next) {
                if next != 0 {
                    out.push(next);
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                if bytes[i] == b'}' {
                    brackets -= 1;
                }
                out.push(bytes[i]);
                i += 1;
            }
        }
        CString::new(out).expect("format unescape output has no NUL")
    }
}
unsafe fn format_strip(ft: &mut format_tree, es: &mut format_expand_state, s: &CStr) -> CString {
    unsafe {
        let mut brackets: core::ffi::c_int = 0 as core::ffi::c_int;
        let bytes = s.to_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if format_check_time(ft, es) == 0 {
                return CString::default();
            }
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            if bytes[i] == b'#' && bytes.get(i + 1) == Some(&b'{') {
                brackets += 1;
            }
            if bytes[i] == b'#' && b",#{}:\0".contains(&next) {
                if brackets != 0 {
                    out.push(bytes[i]);
                }
            } else {
                if bytes[i] == b'}' {
                    brackets -= 1;
                }
                out.push(bytes[i]);
            }
            i += 1;
        }
        CString::new(out).expect("format strip output has no NUL")
    }
}
unsafe fn format_skip1(
    mut context: Option<(&mut format_tree, &mut format_expand_state)>,
    s: &CStr,
    end: &CStr,
) -> Option<usize> {
    unsafe {
        let mut brackets: core::ffi::c_int = 0 as core::ffi::c_int;
        let bytes = s.to_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if context
                .as_mut()
                .is_some_and(|(ft, es)| format_check_time(ft, es) == 0)
            {
                return None;
            }
            if bytes[i] == b'#' && bytes.get(i + 1) == Some(&b'{') {
                brackets += 1;
            }
            if bytes[i] == b'#' && bytes.get(i + 1).is_some_and(|next| b",#{}:".contains(next)) {
                i += 1;
            } else {
                if bytes[i] == b'}' {
                    brackets -= 1;
                }
                if end.to_bytes().contains(&bytes[i]) && brackets == 0 as core::ffi::c_int {
                    return Some(i);
                }
            }
            i += 1;
        }
        None
    }
}
unsafe fn format_choose(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    s: &CStr,
    expand: core::ffi::c_int,
) -> Option<(CString, CString)> {
    unsafe {
        let cp = format_skip1(Some((ft, es)), s, c",")?;
        if expand != 0 {
            let left0 = CString::new(&s.to_bytes()[..cp]).expect("format left operand has no NUL");
            let right0 = CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[cp + 1..])
                .expect("format right operand has no NUL")
                .to_owned();
            let left = format_expand1(ft, es, &left0);
            let right = format_expand1(ft, es, &right0);
            Some((left, right))
        } else {
            let left = CString::new(&s.to_bytes()[..cp]).expect("format left operand has no NUL");
            let right = CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[cp + 1..])
                .expect("format right operand has no NUL")
                .to_owned();
            Some((left, right))
        }
    }
}
pub fn format_true(s: Option<&CStr>) -> core::ffi::c_int {
    s.is_some_and(|s| !s.is_empty() && s.to_bytes() != b"0") as core::ffi::c_int
}
fn format_is_end(c: core::ffi::c_char) -> core::ffi::c_int {
    (c as core::ffi::c_int == ';' as i32 || c as core::ffi::c_int == ':' as i32) as core::ffi::c_int
}
fn format_add_modifier(list: &mut Vec<format_modifier>, c: &[u8], argv: Vec<CString>) {
    assert!(c.len() <= 2);
    let mut modifier = [0; 3];
    modifier[..c.len()].copy_from_slice(c);
    list.push(format_modifier {
        modifier,
        size: c.len() as u_int,
        argv,
    });
}
unsafe fn format_build_modifiers(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    s: &CStr,
) -> Option<(Vec<format_modifier>, usize)> {
    unsafe {
        let bytes = s.to_bytes();
        let mut offset = 0;
        let mut list: Vec<format_modifier> = Vec::new();
        while offset < bytes.len() && bytes[offset] != b':' {
            if bytes[offset] == b';' {
                offset += 1;
            }
            if offset >= bytes.len() {
                break;
            }
            let one = bytes[offset];
            let next = bytes.get(offset + 1).copied().unwrap_or_default();
            let modifier_offset = offset;
            if b"labcdnwETSWPL!<>".contains(&one) && format_is_end(next as _) != 0 {
                format_add_modifier(&mut list, &bytes[offset..offset + 1], Vec::new());
                offset += 1;
            } else if matches!(
                bytes.get(offset..offset + 2),
                Some(b"||" | b"&&" | b"!!" | b"!=" | b"==" | b"<=" | b">=")
            ) && format_is_end(bytes.get(offset + 2).copied().unwrap_or_default() as _)
                != 0
            {
                format_add_modifier(&mut list, &bytes[offset..offset + 2], Vec::new());
                offset += 2;
            } else {
                if !b"mCLNPSst=pReqW".contains(&one) {
                    break;
                }
                if format_is_end(next as _) != 0 {
                    format_add_modifier(&mut list, &bytes[offset..offset + 1], Vec::new());
                    offset += 1;
                } else {
                    let mut argv = Vec::new();
                    let arg_start = offset + 1;
                    if arg_start >= bytes.len() {
                        break;
                    }
                    let arg_cstr =
                        CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[arg_start..]).ok()?;
                    if !arg_cstr
                        .to_bytes()
                        .first()
                        .is_some_and(|c| c.is_ascii_punctuation())
                        || bytes[arg_start] == b'-'
                    {
                        let end_offset = format_skip1(Some((ft, es)), arg_cstr, c":;")?;
                        let end = arg_start + end_offset;
                        argv.push(format_expand1(
                            ft,
                            es,
                            &CString::new(&bytes[arg_start..end]).ok()?,
                        ));
                        format_add_modifier(&mut list, &bytes[offset..offset + 1], argv);
                        offset = end;
                    } else {
                        let delimiter = bytes[arg_start];
                        offset = arg_start;
                        loop {
                            if bytes[offset] == delimiter
                                && format_is_end(
                                    bytes.get(offset + 1).copied().unwrap_or_default() as _
                                ) != 0
                            {
                                offset += 1;
                                break;
                            }
                            if offset + 1 >= bytes.len() {
                                break;
                            }
                            let tail =
                                CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[offset + 1..])
                                    .ok()?;
                            let delim = CString::new([delimiter, b';', b':']).ok()?;
                            let end_offset = format_skip1(Some((ft, es)), tail, &delim)?;
                            let start = offset + 1;
                            let end = start + end_offset;
                            argv.push(format_expand1(
                                ft,
                                es,
                                &CString::new(&bytes[start..end]).ok()?,
                            ));
                            offset = end;
                            if format_is_end(bytes[offset] as _) != 0 {
                                break;
                            }
                        }
                        format_add_modifier(
                            &mut list,
                            &bytes[modifier_offset..modifier_offset + 1],
                            argv,
                        );
                    }
                }
            }
        }
        if bytes.get(offset).copied() != Some(b':') {
            return None;
        }
        Some((list, offset + 1))
    }
}
fn format_match(fm: &format_modifier, pattern: &CStr, text: &CStr) -> CString {
    let modifiers = fm
        .argv
        .first()
        .map_or(&[][..], |argument| argument.to_bytes());
    let matched = if modifiers.contains(&b'r') {
        let flags = REG_EXTENDED
            | REG_NOSUB
            | if modifiers.contains(&b'i') {
                REG_ICASE
            } else {
                0
            };
        crate::CompiledRegex::compile(pattern, flags)
            .is_some_and(|regex| regex.captures::<0>(text, 0, 0).is_some())
    } else {
        let flags = if modifiers.contains(&b'i') {
            FNM_CASEFOLD
        } else {
            0
        };
        unsafe { fnmatch(pattern.as_ptr(), text.as_ptr(), flags) == 0 }
    };
    if matched {
        c"1".to_owned()
    } else {
        c"0".to_owned()
    }
}
fn format_sub(fm: &format_modifier, text: &CStr, pattern: &CStr, with: &CStr) -> CString {
    let mut flags: core::ffi::c_int = REG_EXTENDED;
    if (fm.argv.len() as core::ffi::c_int) >= 3 as core::ffi::c_int
        && fm.argv[2].as_bytes().contains(&b'i')
    {
        flags |= REG_ICASE;
    }
    RustRegsub::substitute(pattern, with, text, flags).unwrap_or_else(|| text.to_owned())
}
unsafe fn format_search(fm: &format_modifier, wp: &impl crate::WindowPane, s: &CStr) -> CString {
    unsafe {
        let flags = fm.argv.first().map_or(&[][..], |flags| flags.as_bytes());
        let ignore = flags.contains(&b'i') as core::ffi::c_int;
        let regex = flags.contains(&b'r') as core::ffi::c_int;
        xasprintf(c"%u", fmt_args![window_pane_search(wp, s, regex, ignore)])
    }
}
unsafe fn format_bool_op_1(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
    not: core::ffi::c_int,
) -> CString {
    unsafe {
        let mut result: core::ffi::c_int;
        let expanded = format_expand1(ft, es, fmt);
        result = format_true(Some(&expanded));
        if not != 0 {
            result = (result == 0) as core::ffi::c_int;
        }
        if result != 0 {
            c"1".to_owned()
        } else {
            c"0".to_owned()
        }
    }
}
unsafe fn format_bool_op_n(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
    and: core::ffi::c_int,
) -> CString {
    unsafe {
        let mut result: core::ffi::c_int;
        result = if and != 0 {
            1 as core::ffi::c_int
        } else {
            0 as core::ffi::c_int
        };
        let mut rest = fmt;
        while if and != 0 {
            result
        } else {
            (result == 0) as core::ffi::c_int
        } != 0
        {
            let cp2 = format_skip1(Some((ft, es)), rest, c",");
            let raw = if let Some(len) = cp2 {
                CString::new(&rest.to_bytes()[..len]).expect("format operand has no NUL")
            } else {
                rest.to_owned()
            };
            let expanded = format_expand1(ft, es, &raw);
            format_log1(
                ft,
                es,
                c"format_bool_op_n",
                c"operator %s has operand: %s",
                fmt_args![if and != 0 { c"&&" } else { c"||" }, expanded.as_c_str()],
            );
            if and != 0 {
                result = (result != 0 && format_true(Some(&expanded)) != 0) as core::ffi::c_int;
            } else {
                result = (result != 0 || format_true(Some(&expanded)) != 0) as core::ffi::c_int;
            }
            let Some(cp2) = cp2 else {
                break;
            };
            rest = CStr::from_bytes_with_nul(&rest.to_bytes_with_nul()[cp2 + 1..])
                .expect("format operand suffix retains the input terminator");
        }
        if result != 0 {
            c"1".to_owned()
        } else {
            c"0".to_owned()
        }
    }
}
unsafe fn format_session_name(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
) -> Option<CString> {
    let SESSIONS = SESSIONS_FIELD.get();

    unsafe {
        let name = format_expand1(ft, es, fmt);
        for s in SESSIONS.read().values() {
            if s.name().as_deref() == Some(name.as_c_str()) {
                return Some(c"1".to_owned());
            }
        }
        Some(c"0".to_owned())
    }
}
unsafe fn format_loop_sessions(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let c = (*ft).client();
        let item = (*ft).item();
        let mut next;
        let (all, active) = format_choose(ft, es, fmt, 0 as core::ffi::c_int)
            .map(|(all, active)| (all, Some(active)))
            .unwrap_or_else(|| (fmt.to_owned(), None));
        let mut value = Vec::new();
        let l = sort_get_sessions(sc);
        let n = l.len();
        for (i, s) in l.iter().enumerate() {
            format_log1(
                ft,
                es,
                c"format_loop_sessions",
                c"session loop: $%u",
                fmt_args![s.id()],
            );
            let drawn = (*ft).drawn_client();
            let use_0 = if let Some(drawn) = drawn.as_ref()
                && active.is_some()
                && drawn
                    .attached_session()
                    .is_some_and(|attached| s.id() == attached.id())
            {
                active.as_ref().unwrap()
            } else {
                &all
            };
            let mut last = 0 as core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let mut nft = match item.as_ref() {
                Some(item) => item.with_item(|item| {
                    format_create_for_client(c.as_ref(), Some(item), FORMAT_NONE, ft.flags | last)
                }),
                None => format_create_for_client(c.as_ref(), None, FORMAT_NONE, ft.flags | last),
            };
            format_defaults_for_client(
                &mut nft,
                drawn.as_ref(),
                Some(s.as_session()),
                None,
                None::<&crate::types::window_pane>,
            );
            next = es.clone();
            next.flags |= 0 as core::ffi::c_int;
            let expanded = format_expand1(&mut nft, &mut next, use_0);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format session loop output has no NUL"))
    }
}
unsafe fn format_window_name(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
) -> Option<CString> {
    unsafe {
        let Some(session) = ft.session() else {
            format_log1(
                ft,
                es,
                c"format_window_name",
                c"window name but no session",
                fmt_args![],
            );
            return None;
        };
        let name = format_expand1(ft, es, fmt);
        let found = session.as_session().windows.values().any(|link| {
            link.window_handle()
                .is_some_and(|window| window.window_name().as_deref() == Some(name.as_c_str()))
        });
        Some(if found {
            c"1".to_owned()
        } else {
            c"0".to_owned()
        })
    }
}
unsafe fn format_add_window_neighbor(
    nft: &mut format_tree,
    wl: &winlink,
    s: &session,
    prefix: &CStr,
) {
    unsafe {
        let key = xasprintf(c"%s_window_index", fmt_args![prefix]);
        format_add(nft, &key, c"%u", fmt_args![wl.idx]);
        let key = xasprintf(c"%s_window_active", fmt_args![prefix]);
        format_add(
            nft,
            &key,
            c"%d",
            fmt_args![s.curw().is_some_and(|current| current.idx == wl.idx) as core::ffi::c_int],
        );
        let options = wl.window_handle().expect("a link has a window").options();
        for name in options.local_names() {
            if name.to_bytes().starts_with(b"@") {
                options.with_entry(&name, true, |entry| {
                    if let Some(entry) = entry {
                        let prefixed = xasprintf(c"%s_%s", fmt_args![prefix, name.as_c_str()]);
                        let value = RustOptionsEngine.display(entry, -1, 1);
                        format_add(nft, &prefixed, c"%s", fmt_args![value.as_c_str()]);
                    }
                });
            }
        }
    }
}
unsafe fn format_loop_windows(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let c = (*ft).client();
        let item = (*ft).item();
        let mut next;
        let Some(s) = (*ft).session() else {
            format_log1(
                ft,
                es,
                c"format_loop_windows",
                c"window loop but no session",
                fmt_args![],
            );
            return None;
        };
        let (all, active) = format_choose(ft, es, fmt, 0 as core::ffi::c_int)
            .map(|(all, active)| (all, Some(active)))
            .unwrap_or_else(|| (fmt.to_owned(), None));
        let mut value = Vec::new();
        let l = s.sorted_winlinks(sc);
        let n = l.len();
        for (i, link) in l.iter().enumerate() {
            let Some(wl) = link.get() else {
                continue;
            };
            let w = wl.window_handle().expect("a link has a window");
            format_log1(
                ft,
                es,
                c"format_loop_windows",
                c"window loop: %u @%u",
                fmt_args![link.index(), w.window_id()],
            );
            let use_0 = if let Some(active) = active.as_ref()
                && s.curw()
                    .is_some_and(|current| current.index() == link.index())
            {
                active
            } else {
                &all
            };
            let mut last = 0 as core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let tag = (FORMAT_WINDOW | w.window_id()) as core::ffi::c_int;
            let mut nft = match item.as_ref() {
                Some(item) => item.with_item(|item| {
                    format_create_for_client(c.as_ref(), Some(item), tag, ft.flags | last)
                }),
                None => format_create_for_client(c.as_ref(), None, tag, ft.flags | last),
            };
            format_defaults_for_client(
                &mut nft,
                ft.drawn_client().as_ref(),
                (*ft)
                    .session()
                    .as_ref()
                    .map(|reference| reference.as_session()),
                Some(wl),
                None::<&crate::types::window_pane>,
            );
            let current_index = s.curw().map(|current| current.index());
            format_add(
                &mut nft,
                c"window_after_active",
                c"%d",
                fmt_args![
                    (i > 0
                        && l[i - 1]
                            .get()
                            .is_some_and(|wl| Some(wl.idx) == current_index))
                        as core::ffi::c_int
                ],
            );
            format_add(
                &mut nft,
                c"window_before_active",
                c"%d",
                fmt_args![
                    (i + 1 < n
                        && l[i + 1]
                            .get()
                            .is_some_and(|wl| Some(wl.idx) == current_index))
                        as core::ffi::c_int
                ],
            );
            if i + 1 < n
                && let Some(neighbor) = l[i + 1].get()
            {
                format_add_window_neighbor(&mut nft, neighbor, s.as_session(), c"next");
            }
            if i > 0
                && let Some(neighbor) = l[i - 1].get()
            {
                format_add_window_neighbor(&mut nft, neighbor, s.as_session(), c"prev");
            }
            next = es.clone();
            next.flags |= 0 as core::ffi::c_int;
            let expanded = format_expand1(&mut nft, &mut next, use_0);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format window loop output has no NUL"))
    }
}
unsafe fn format_loop_panes(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let c = (*ft).client();
        let item = (*ft).item();
        let mut next;
        let Some(w) = (*ft).window() else {
            format_log1(
                ft,
                es,
                c"format_loop_panes",
                c"pane loop but no window",
                fmt_args![],
            );
            return None;
        };
        let (all, active) = format_choose(ft, es, fmt, 0 as core::ffi::c_int)
            .map(|(all, active)| (all, Some(active)))
            .unwrap_or_else(|| (fmt.to_owned(), None));
        let mut value = Vec::new();
        let l = w.sorted_panes(sc);
        let n = l.len();
        for (i, wp) in l.into_iter().enumerate() {
            if wp.get().is_none() {
                continue;
            }
            format_log1(
                ft,
                es,
                c"format_loop_panes",
                c"pane loop: %%%u",
                fmt_args![wp.id()],
            );
            let use_0 = if let Some(active) = active.as_ref()
                && w.active_pane_id() == Some(wp.id())
            {
                active
            } else {
                &all
            };
            let mut last = 0 as core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let tag = (FORMAT_PANE | wp.id()) as core::ffi::c_int;
            let mut nft = match item.as_ref() {
                Some(item) => item.with_item(|item| {
                    format_create_for_client(c.as_ref(), Some(item), tag, ft.flags | last)
                }),
                None => format_create_for_client(c.as_ref(), None, tag, ft.flags | last),
            };
            let winlink = (*ft).winlink();
            format_defaults_for_client(
                &mut nft,
                ft.drawn_client().as_ref(),
                (*ft)
                    .session()
                    .as_ref()
                    .map(|reference| reference.as_session()),
                winlink.as_ref().and_then(WinlinkRef::get),
                wp.get(),
            );
            next = es.clone();
            next.flags |= 0 as core::ffi::c_int;
            let expanded = format_expand1(&mut nft, &mut next, use_0);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format pane loop output has no NUL"))
    }
}
unsafe fn format_loop_clients(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let item = (*ft).item();
        let mut next;
        let mut value = Vec::new();
        let l = sort_get_clients(sc);
        let n = l.len();
        for (i, c) in l.iter().enumerate() {
            format_log1(
                ft,
                es,
                c"format_loop_clients",
                c"client loop: %s",
                fmt_args![c.name()],
            );
            let mut last = 0 as core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let mut nft = match item.as_ref() {
                Some(item) => item.with_item(|item| {
                    format_create_for_client(
                        Some(c),
                        Some(item),
                        0 as core::ffi::c_int,
                        ft.flags | last,
                    )
                }),
                None => {
                    format_create_for_client(Some(c), None, 0 as core::ffi::c_int, ft.flags | last)
                }
            };
            let winlink = ft.winlink();
            let pane = ft.pane_handle();
            format_defaults_for_client(
                &mut nft,
                Some(c),
                (*ft)
                    .session()
                    .as_ref()
                    .map(|reference| reference.as_session()),
                winlink.as_ref().and_then(WinlinkRef::get),
                pane.as_ref().and_then(|pane| pane.get()),
            );
            next = es.clone();
            next.flags |= 0 as core::ffi::c_int;
            let expanded = format_expand1(&mut nft, &mut next, fmt);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format client loop output has no NUL"))
    }
}
unsafe fn format_replace_expression(
    ft: &mut format_tree,
    mexp: &format_modifier,
    es: &mut format_expand_state,
    copy: &CStr,
) -> Option<CString> {
    unsafe {
        let operator_text = mexp.argv.first()?;
        let operator = match operator_text.as_bytes() {
            b"+" => ADD,
            b"-" => SUBTRACT,
            b"*" => MULTIPLY,
            b"/" => DIVIDE,
            b"%" | b"m" => MODULUS,
            b"==" => EQUAL,
            b"!=" => NOT_EQUAL,
            b">" => GREATER_THAN,
            b"<" => LESS_THAN,
            b">=" => GREATER_THAN_EQUAL,
            b"<=" => LESS_THAN_EQUAL,
            _ => {
                format_log1(
                    ft,
                    es,
                    c"format_replace_expression",
                    c"expression has no valid operator: '%s'",
                    fmt_args![operator_text.as_c_str()],
                );
                return None;
            }
        };
        let use_fp = mexp
            .argv
            .get(1)
            .is_some_and(|flags| flags.as_bytes().contains(&b'f'));
        let mut prec = if use_fp { 2 } else { 0 } as u_int;
        if let Some(precision) = mexp.argv.get(2) {
            match strtonum(
                precision,
                -FORMAT_MAX_PRECISION as core::ffi::c_longlong,
                FORMAT_MAX_PRECISION as core::ffi::c_longlong,
            ) {
                Ok(value) => prec = value as u_int,
                Err(errstr) => {
                    format_log1(
                        ft,
                        es,
                        c"format_replace_expression",
                        c"expression precision %s: %s",
                        fmt_args![errstr, precision.as_c_str()],
                    );
                    return None;
                }
            }
        }
        let Some((left, right)) = format_choose(ft, es, copy, 1) else {
            format_log1(
                ft,
                es,
                c"format_replace_expression",
                c"expression syntax error",
                fmt_args![],
            );
            return None;
        };
        let Some(mut mleft) = crate::compat::strtod_complete(&left) else {
            format_log1(
                ft,
                es,
                c"format_replace_expression",
                c"expression left side is invalid: %s",
                fmt_args![left.as_c_str()],
            );
            return None;
        };
        let Some(mut mright) = crate::compat::strtod_complete(&right) else {
            format_log1(
                ft,
                es,
                c"format_replace_expression",
                c"expression right side is invalid: %s",
                fmt_args![right.as_c_str()],
            );
            return None;
        };
        if !use_fp {
            mleft = mleft as core::ffi::c_longlong as core::ffi::c_double;
            mright = mright as core::ffi::c_longlong as core::ffi::c_double;
        }
        format_log1(
            ft,
            es,
            c"format_replace_expression",
            c"expression left side is: %.*f",
            fmt_args![prec, mleft],
        );
        format_log1(
            ft,
            es,
            c"format_replace_expression",
            c"expression right side is: %.*f",
            fmt_args![prec, mright],
        );
        let result = match operator {
            ADD => mleft + mright,
            SUBTRACT => mleft - mright,
            MULTIPLY => mleft * mright,
            DIVIDE => mleft / mright,
            MODULUS => fmod(mleft, mright),
            EQUAL => (fabs(mleft - mright) < 1e-9f64) as core::ffi::c_int as core::ffi::c_double,
            NOT_EQUAL => {
                (fabs(mleft - mright) > 1e-9f64) as core::ffi::c_int as core::ffi::c_double
            }
            GREATER_THAN => (mleft > mright) as core::ffi::c_int as core::ffi::c_double,
            GREATER_THAN_EQUAL => (mleft >= mright) as core::ffi::c_int as core::ffi::c_double,
            LESS_THAN => (mleft < mright) as core::ffi::c_int as core::ffi::c_double,
            LESS_THAN_EQUAL => (mleft <= mright) as core::ffi::c_int as core::ffi::c_double,
            _ => unreachable!("operator was checked above"),
        };
        let result = if use_fp {
            result
        } else {
            result as core::ffi::c_longlong as core::ffi::c_double
        };
        let value = xasprintf(c"%.*f", fmt_args![prec, result]);
        format_log1(
            ft,
            es,
            c"format_replace_expression",
            c"expression result is %s",
            fmt_args![value.as_c_str()],
        );
        Some(value)
    }
}
unsafe fn format_replace(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    key: &[u8],
    out: &mut Vec<u8>,
) -> core::ffi::c_int {
    unsafe {
        let current_block: u64;
        let mut sort_crit = sort_criteria_t::default();
        let pane = ft.pane_handle();
        let mut cp2: Option<usize>;
        let mut marker: Option<&CStr> = None;
        let mut time_format: Option<CString> = None;
        let mut value: Option<CString> = None;
        let mut modifiers: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut limit: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut width: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut j: core::ffi::c_int;
        let mut c: core::ffi::c_int;
        let mut cmp: Option<usize> = None;
        let mut search: Option<usize> = None;
        let mut sub: Vec<usize> = Vec::new();
        let mut mexp: Option<usize> = None;
        let mut bool_op_n: Option<usize> = None;
        let mut i: u_int;

        let mut nsub: u_int = 0 as u_int;
        let mut nrep: u_int = 0;
        let mut next;
        sort_crit.set_order(SORT_ORDER);
        sort_crit.set_reversed(false);
        let copy0 = CString::new(key).expect("format replacement key has no NUL");
        let (list, consumed) = format_build_modifiers(ft, es, &copy0).unwrap_or_default();
        let copy = CStr::from_bytes_with_nul(&copy0.as_bytes_with_nul()[consumed..])
            .expect("format replacement suffix retains the input terminator");
        let count: u_int = list.len() as u_int;
        i = 0 as u_int;
        while i < count {
            let fm = &list[i as usize];
            if format_logging(&mut *ft) != 0 {
                format_log1(
                    ft,
                    es,
                    c"format_replace",
                    c"modifier %u is %s",
                    fmt_args![i, fm.format_modifier_name()],
                );
                j = 0 as core::ffi::c_int;
                while j < (fm.argv.len() as core::ffi::c_int) {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"modifier %u argument %d: %s",
                        fmt_args![i, j, fm.argv[j as usize].as_c_str()],
                    );
                    j += 1;
                }
            }
            if fm.size == 1 as u_int {
                match fm.modifier[0 as core::ffi::c_int as usize] as core::ffi::c_int {
                    109 | 60 | 62 => {
                        cmp = Some(i as usize);
                    }
                    33 => {
                        modifiers |= FORMAT_NOT;
                    }
                    67 => {
                        search = Some(i as usize);
                    }
                    115 => {
                        if !((fm.argv.len() as core::ffi::c_int) < 2 as core::ffi::c_int) {
                            sub.push(i as usize);
                            nsub = nsub.wrapping_add(1);
                        }
                    }
                    61 => {
                        if !((fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int) {
                            limit = strtonum(
                                &fm.argv[0],
                                -FORMAT_MAX_WIDTH as core::ffi::c_longlong,
                                FORMAT_MAX_WIDTH as core::ffi::c_longlong,
                            )
                            .map_or(0, |value| value as core::ffi::c_int);
                            if (fm.argv.len() as core::ffi::c_int) >= 2 as core::ffi::c_int {
                                marker = Some(fm.argv[1].as_c_str());
                            }
                        }
                    }
                    112 => {
                        if !((fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int) {
                            width = strtonum(
                                &fm.argv[0],
                                -FORMAT_MAX_WIDTH as core::ffi::c_longlong,
                                FORMAT_MAX_WIDTH as core::ffi::c_longlong,
                            )
                            .map_or(0, |value| value as core::ffi::c_int);
                        }
                    }
                    119 => {
                        modifiers |= FORMAT_WIDTH;
                    }
                    101 => {
                        if !((fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int
                            || (fm.argv.len() as core::ffi::c_int) > 3 as core::ffi::c_int)
                        {
                            mexp = Some(i as usize);
                        }
                    }
                    108 => {
                        modifiers |= FORMAT_LITERAL;
                    }
                    97 => {
                        modifiers |= FORMAT_CHARACTER;
                    }
                    98 => {
                        modifiers |= FORMAT_BASENAME;
                    }
                    99 => {
                        modifiers |= FORMAT_COLOUR;
                    }
                    100 => {
                        modifiers |= FORMAT_DIRNAME;
                    }
                    110 => {
                        modifiers |= FORMAT_LENGTH;
                    }
                    116 => {
                        modifiers |= FORMAT_TIMESTRING;
                        if !((fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int) {
                            if fm.argv[0].as_bytes().contains(&b'p') {
                                modifiers |= FORMAT_PRETTY;
                            } else if (fm.argv.len() as core::ffi::c_int) >= 2 as core::ffi::c_int
                                && fm.argv[0].as_bytes().contains(&b'f')
                            {
                                time_format = Some(format_strip(ft, es, &fm.argv[1]));
                            }
                        }
                    }
                    113 => {
                        if (fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int {
                            modifiers |= FORMAT_QUOTE_SHELL;
                        } else if fm.argv[0].as_bytes().contains(&b'e')
                            || fm.argv[0].as_bytes().contains(&b'h')
                        {
                            modifiers |= FORMAT_QUOTE_STYLE;
                        } else if fm.argv[0].as_bytes().contains(&b'a') {
                            modifiers |= FORMAT_QUOTE_ARGUMENTS;
                        }
                    }
                    69 => {
                        modifiers |= FORMAT_EXPAND;
                    }
                    84 => {
                        modifiers |= FORMAT_EXPANDTIME;
                    }
                    78 => {
                        if (fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int
                            || fm.argv[0].as_bytes().contains(&b'w')
                        {
                            modifiers |= FORMAT_WINDOW_NAME;
                        } else if fm.argv[0].as_bytes().contains(&b's') {
                            modifiers |= FORMAT_SESSION_NAME;
                        }
                    }
                    83 => {
                        modifiers |= FORMAT_SESSIONS;
                        if (fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int {
                            sort_crit.set_order(SORT_INDEX);
                            sort_crit.set_reversed(false);
                        } else {
                            if fm.argv[0].as_bytes().contains(&b'i') {
                                sort_crit.set_order(SORT_INDEX);
                            } else if fm.argv[0].as_bytes().contains(&b'n') {
                                sort_crit.set_order(SORT_NAME);
                            } else if fm.argv[0].as_bytes().contains(&b't') {
                                sort_crit.set_order(SORT_ACTIVITY);
                            } else {
                                sort_crit.set_order(SORT_INDEX);
                            }
                            if fm.argv[0].as_bytes().contains(&b'r') {
                                sort_crit.set_reversed(true);
                            } else {
                                sort_crit.set_reversed(false);
                            }
                        }
                    }
                    87 => {
                        modifiers |= FORMAT_WINDOWS;
                        if (fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int {
                            sort_crit.set_order(SORT_ORDER);
                            sort_crit.set_reversed(false);
                        } else {
                            if fm.argv[0].as_bytes().contains(&b'i') {
                                sort_crit.set_order(SORT_ORDER);
                            } else if fm.argv[0].as_bytes().contains(&b'n') {
                                sort_crit.set_order(SORT_NAME);
                            } else if fm.argv[0].as_bytes().contains(&b't') {
                                sort_crit.set_order(SORT_ACTIVITY);
                            } else {
                                sort_crit.set_order(SORT_ORDER);
                            }
                            if fm.argv[0].as_bytes().contains(&b'r') {
                                sort_crit.set_reversed(true);
                            } else {
                                sort_crit.set_reversed(false);
                            }
                        }
                    }
                    80 => {
                        modifiers |= FORMAT_PANES;
                        sort_crit.set_order(SORT_CREATION);
                        if (fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int {
                            sort_crit.set_reversed(false);
                        } else if fm.argv[0].as_bytes().contains(&b'r') {
                            sort_crit.set_reversed(true);
                        } else {
                            sort_crit.set_reversed(false);
                        }
                    }
                    76 => {
                        modifiers |= FORMAT_CLIENTS;
                        if (fm.argv.len() as core::ffi::c_int) < 1 as core::ffi::c_int {
                            sort_crit.set_order(SORT_ORDER);
                            sort_crit.set_reversed(false);
                        } else {
                            if fm.argv[0].as_bytes().contains(&b'i') {
                                sort_crit.set_order(SORT_ORDER);
                            } else if fm.argv[0].as_bytes().contains(&b'n') {
                                sort_crit.set_order(SORT_NAME);
                            } else if fm.argv[0].as_bytes().contains(&b't') {
                                sort_crit.set_order(SORT_ACTIVITY);
                            } else {
                                sort_crit.set_order(SORT_ORDER);
                            }
                            if fm.argv[0].as_bytes().contains(&b'r') {
                                sort_crit.set_reversed(true);
                            } else {
                                sort_crit.set_reversed(false);
                            }
                        }
                    }
                    82 => {
                        modifiers |= FORMAT_REPEAT;
                    }
                    _ => {}
                }
            } else if fm.size == 2 as u_int {
                if fm.format_modifier_name() == c"||" || fm.format_modifier_name() == c"&&" {
                    bool_op_n = Some(i as usize);
                } else if fm.format_modifier_name() == c"!!" {
                    modifiers |= FORMAT_NOT_NOT;
                } else if fm.format_modifier_name() == c"=="
                    || fm.format_modifier_name() == c"!="
                    || fm.format_modifier_name() == c">="
                    || fm.format_modifier_name() == c"<="
                {
                    cmp = Some(i as usize);
                }
            }
            i = i.wrapping_add(1);
        }
        let cmp = cmp.map(|i| &list[i]);
        let search = search.map(|i| &list[i]);
        let mexp = mexp.map(|i| &list[i]);
        let bool_op_n = bool_op_n.map(|i| &list[i]);
        let sub: Vec<&format_modifier> = sub.into_iter().map(|i| &list[i]).collect();
        if modifiers & FORMAT_LITERAL != 0 {
            format_log1(
                ft,
                es,
                c"format_replace",
                c"literal string is '%s'",
                fmt_args![copy],
            );
            value = Some(format_unescape(ft, es, copy));
        } else if modifiers & FORMAT_CHARACTER != 0 {
            let new = format_expand1(ft, es, copy);
            value = match strtonum(
                &new,
                32 as core::ffi::c_longlong,
                126 as core::ffi::c_longlong,
            ) {
                Ok(value) => {
                    c = value as core::ffi::c_int;
                    Some(xasprintf(c"%c", fmt_args![c]))
                }
                Err(_) => Some(CString::default()),
            };
        } else if modifiers & FORMAT_COLOUR != 0 {
            let new = format_expand1(ft, es, copy);
            c = RustColourEngine.from_string(&new);
            if c == -(1 as core::ffi::c_int) || {
                c = RustColourEngine.force_rgb(c);
                c == -(1 as core::ffi::c_int)
            } {
                value = Some(CString::default());
            } else {
                value = Some(xasprintf(
                    c"%06x",
                    fmt_args![c & 0xffffff as core::ffi::c_int],
                ));
            }
        } else {
            if modifiers & FORMAT_SESSIONS != 0 {
                value = format_loop_sessions(ft, es, copy, &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_WINDOWS != 0 {
                value = format_loop_windows(ft, es, copy, &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_PANES != 0 {
                value = format_loop_panes(ft, es, copy, &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_CLIENTS != 0 {
                value = format_loop_clients(ft, es, copy, &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_WINDOW_NAME != 0 {
                value = format_window_name(ft, es, copy);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_SESSION_NAME != 0 {
                value = format_session_name(ft, es, copy);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if let Some(search) = search {
                let new = format_expand1(ft, es, copy);
                if let Some(wp) = pane.as_ref().and_then(|pane| pane.get()) {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"search '%s' pane %%%u",
                        fmt_args![new.as_c_str(), wp.pane_id()],
                    );
                    value = Some(format_search(search, wp, &new));
                } else {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"search '%s' but no pane",
                        fmt_args![new.as_c_str()],
                    );
                    value = Some(c"0".to_owned());
                }
                current_block = 4781510679662115254;
            } else if modifiers & FORMAT_REPEAT != 0 {
                if let Some((left, right)) = format_choose(ft, es, copy, 1 as core::ffi::c_int) {
                    let parsed = strtonum(
                        &right,
                        1 as core::ffi::c_longlong,
                        FORMAT_MAX_REPEAT as core::ffi::c_longlong,
                    );
                    if let Ok(parsed) = parsed {
                        nrep = parsed as u_int;
                    }
                    if parsed.is_err() {
                        value = Some(CString::default());
                        current_block = 4781510679662115254;
                    } else {
                        let mut repeated = Vec::new();
                        let mut failed = false;
                        i = 0 as u_int;
                        while i < nrep {
                            if format_check_time(ft, es) == 0 {
                                failed = true;
                                break;
                            }
                            repeated.extend_from_slice(left.as_bytes());
                            i = i.wrapping_add(1);
                        }
                        if failed {
                            current_block = 75153483021275631;
                        } else {
                            value = Some(
                                CString::new(repeated).expect("format repeat output has no NUL"),
                            );
                            current_block = 4781510679662115254;
                        }
                    }
                } else {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"repeat syntax error: %s",
                        fmt_args![copy],
                    );
                    current_block = 75153483021275631;
                }
            } else if modifiers & FORMAT_NOT != 0 {
                value = Some(format_bool_op_1(ft, es, copy, 1 as core::ffi::c_int));
                current_block = 4781510679662115254;
            } else if modifiers & FORMAT_NOT_NOT != 0 {
                value = Some(format_bool_op_1(ft, es, copy, 0 as core::ffi::c_int));
                current_block = 4781510679662115254;
            } else if let Some(bool_op_n) = bool_op_n {
                if bool_op_n.format_modifier_name() == c"||" {
                    value = Some(format_bool_op_n(ft, es, copy, 0 as core::ffi::c_int));
                } else if bool_op_n.format_modifier_name() == c"&&" {
                    value = Some(format_bool_op_n(ft, es, copy, 1 as core::ffi::c_int));
                }
                current_block = 4781510679662115254;
            } else if let Some(cmp) = cmp {
                if let Some((left, right)) = format_choose(ft, es, copy, 1 as core::ffi::c_int) {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"compare %s left is: %s",
                        fmt_args![cmp.format_modifier_name(), left.as_c_str()],
                    );
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"compare %s right is: %s",
                        fmt_args![cmp.format_modifier_name(), right.as_c_str()],
                    );
                    if cmp.format_modifier_name() == c"m" {
                        value = Some(format_match(cmp, &left, &right));
                    } else {
                        let comparison = if cmp.format_modifier_name() == c"==" {
                            left.as_bytes() == right.as_bytes()
                        } else if cmp.format_modifier_name() == c"!=" {
                            left.as_bytes() != right.as_bytes()
                        } else if cmp.format_modifier_name() == c"<" {
                            left.as_bytes() < right.as_bytes()
                        } else if cmp.format_modifier_name() == c">" {
                            left.as_bytes() > right.as_bytes()
                        } else if cmp.format_modifier_name() == c"<=" {
                            left.as_bytes() <= right.as_bytes()
                        } else if cmp.format_modifier_name() == c">=" {
                            left.as_bytes() >= right.as_bytes()
                        } else {
                            false
                        };
                        value = Some(if comparison {
                            c"1".to_owned()
                        } else {
                            c"0".to_owned()
                        });
                    }
                    current_block = 4781510679662115254;
                } else {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"compare %s syntax error: %s",
                        fmt_args![cmp.format_modifier_name(), copy],
                    );
                    current_block = 75153483021275631;
                }
            } else {
                if copy.to_bytes().first() == Some(&b'?') {
                    let conditional = CStr::from_bytes_with_nul(&copy.to_bytes_with_nul()[1..])
                        .expect("format conditional retains the input terminator");
                    let mut cp = conditional;
                    loop {
                        cp2 = format_skip1(Some((ft, es)), cp, c",");
                        if cp2.is_none() {
                            format_log1(
                                ft,
                                es,
                                c"format_replace",
                                c"no condition matched in '%s'; using last arg",
                                fmt_args![conditional],
                            );
                            value = Some(format_expand1(ft, es, cp));
                            break;
                        } else {
                            let condition = CString::new(&cp.to_bytes()[..cp2.unwrap()])
                                .expect("format condition has no NUL");
                            format_log1(
                                ft,
                                es,
                                c"format_replace",
                                c"condition is: %s",
                                fmt_args![condition.as_c_str()],
                            );
                            let time_format_ptr = time_format.as_deref();
                            let mut found =
                                format_find(&mut *ft, &condition, modifiers, time_format_ptr);
                            if found.is_none() {
                                let expanded = format_expand1(ft, es, &condition);
                                if expanded == condition {
                                    found = Some(CString::default());
                                    format_log1(
                                        ft,
                                        es,
                                        c"format_replace",
                                        c"condition '%s' not found; assuming false",
                                        fmt_args![condition.as_c_str()],
                                    );
                                } else {
                                    found = Some(expanded);
                                }
                            } else if let Some(found) = found.as_ref() {
                                format_log1(
                                    ft,
                                    es,
                                    c"format_replace",
                                    c"condition '%s' found: %s",
                                    fmt_args![condition.as_c_str(), found.as_c_str()],
                                );
                            }
                            cp = CStr::from_bytes_with_nul(
                                &cp.to_bytes_with_nul()[cp2.unwrap() + 1..],
                            )
                            .expect("format conditional suffix retains the input terminator");
                            cp2 = format_skip1(Some((ft, es)), cp, c",");
                            if format_true(found.as_deref()) != 0 {
                                format_log1(
                                    ft,
                                    es,
                                    c"format_replace",
                                    c"condition '%s' is true",
                                    fmt_args![condition.as_c_str()],
                                );
                                if let Some(len) = cp2 {
                                    let right = CString::new(&cp.to_bytes()[..len])
                                        .expect("format conditional result has no NUL");
                                    value = Some(format_expand1(ft, es, &right));
                                } else {
                                    value = Some(format_expand1(ft, es, cp));
                                }
                                break;
                            } else {
                                format_log1(
                                    ft,
                                    es,
                                    c"format_replace",
                                    c"condition '%s' is false",
                                    fmt_args![condition.as_c_str()],
                                );
                                if let Some(len) = cp2 {
                                    cp = CStr::from_bytes_with_nul(
                                        &cp.to_bytes_with_nul()[len + 1..],
                                    )
                                    .expect(
                                        "format conditional suffix retains the input terminator",
                                    );
                                } else {
                                    format_log1(
                                        ft,
                                        es,
                                        c"format_replace",
                                        c"no condition matched in '%s'; using empty string",
                                        fmt_args![conditional],
                                    );
                                    value = Some(CString::default());
                                    break;
                                }
                            }
                        }
                    }
                } else if let Some(mexp) = mexp {
                    value = format_replace_expression(ft, mexp, es, copy);
                    if value.is_none() {
                        value = Some(CString::default());
                    }
                } else if copy.to_bytes().windows(2).any(|bytes| bytes == b"#{") {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"expanding inner format '%s'",
                        fmt_args![copy],
                    );
                    value = Some(format_expand1(ft, es, copy));
                } else {
                    let time_format_ptr = time_format.as_deref();
                    if let Some(result) = format_find(&mut *ft, copy, modifiers, time_format_ptr) {
                        format_log1(
                            ft,
                            es,
                            c"format_replace",
                            c"format '%s' found: %s",
                            fmt_args![copy, result.as_c_str()],
                        );
                        value = Some(result);
                    } else {
                        format_log1(
                            ft,
                            es,
                            c"format_replace",
                            c"format '%s' not found",
                            fmt_args![copy],
                        );
                        value = Some(CString::default());
                    }
                }
                current_block = 4781510679662115254;
            }
            match current_block {
                4781510679662115254 => {}
                _ => {
                    format_log1(
                        ft,
                        es,
                        c"format_replace",
                        c"failed %s",
                        fmt_args![copy0.as_c_str()],
                    );
                    return -(1 as core::ffi::c_int);
                }
            }
        }
        let mut value = value.expect("format replacement has no result");
        if modifiers & FORMAT_EXPAND != 0 {
            value = format_expand1(ft, es, &value);
        } else if modifiers & FORMAT_EXPANDTIME != 0 {
            next = es.clone();
            next.flags |= FORMAT_EXPAND_TIME;
            value = format_expand1(ft, &mut next, &value);
        }
        i = 0 as u_int;
        while i < nsub {
            let left = format_expand1(ft, es, &sub[i as usize].argv[0]);
            let right = format_expand1(ft, es, &sub[i as usize].argv[1]);
            let result = format_sub(sub[i as usize], &value, &left, &right);
            format_log1(
                ft,
                es,
                c"format_replace",
                c"substitute '%s' to '%s': %s",
                fmt_args![left.as_c_str(), right.as_c_str(), result.as_c_str()],
            );
            value = result;
            i = i.wrapping_add(1);
        }
        if limit > 0 as core::ffi::c_int {
            let trimmed = RustFormatText.trim_left(value.as_bytes(), limit as u_int);
            if let Some(marker) = marker
                && trimmed.as_bytes() != value.as_bytes()
            {
                value = xasprintf(c"%s%s", fmt_args![trimmed.as_c_str(), marker]);
            } else {
                value = trimmed;
            }
            format_log1(
                ft,
                es,
                c"format_replace",
                c"applied length limit %d: %s",
                fmt_args![limit, value.as_c_str()],
            );
        } else if limit < 0 as core::ffi::c_int {
            let trimmed = RustFormatText.trim_right(value.as_bytes(), -limit as u_int);
            if let Some(marker) = marker
                && trimmed.as_bytes() != value.as_bytes()
            {
                value = xasprintf(c"%s%s", fmt_args![marker, trimmed.as_c_str()]);
            } else {
                value = trimmed;
            }
            format_log1(
                ft,
                es,
                c"format_replace",
                c"applied length limit %d: %s",
                fmt_args![limit, value.as_c_str()],
            );
        }
        if width > 0 as core::ffi::c_int {
            value = RustUtf8VisModel.pad_right(&value, width as u_int);
            format_log1(
                ft,
                es,
                c"format_replace",
                c"applied padding width %d: %s",
                fmt_args![width, value.as_c_str()],
            );
        } else if width < 0 as core::ffi::c_int {
            value = RustUtf8VisModel.pad_left(&value, -width as u_int);
            format_log1(
                ft,
                es,
                c"format_replace",
                c"applied padding width %d: %s",
                fmt_args![width, value.as_c_str()],
            );
        }
        if modifiers & FORMAT_LENGTH != 0 {
            value = xasprintf(c"%zu", fmt_args![value.as_bytes().len()]);
            format_log1(
                ft,
                es,
                c"format_replace",
                c"replacing with length: %s",
                fmt_args![value.as_c_str()],
            );
        }
        if modifiers & FORMAT_WIDTH != 0 {
            value = xasprintf(c"%u", fmt_args![RustFormatText.width(value.as_bytes())]);
            format_log1(
                ft,
                es,
                c"format_replace",
                c"replacing with width: %s",
                fmt_args![value.as_c_str()],
            );
        }
        out.extend_from_slice(value.as_bytes());
        format_log1(
            ft,
            es,
            c"format_replace",
            c"replaced '%s' with '%s'",
            fmt_args![copy0.as_c_str(), value.as_c_str()],
        );
        0 as core::ffi::c_int
    }
}
unsafe fn format_expand1(
    ft: &mut format_tree,
    es: &mut format_expand_state,
    fmt: &CStr,
) -> CString {
    unsafe {
        let mut buf: Vec<u8> = Vec::with_capacity(64);
        if fmt.is_empty() || format_check_time(ft, es) == 0 {
            return CString::default();
        }
        if es.r#loop == FORMAT_LOOP_LIMIT as u_int {
            format_log1(
                ft,
                es,
                c"format_expand1",
                c"reached loop limit (%u)",
                fmt_args![FORMAT_LOOP_LIMIT],
            );
            return CString::default();
        }
        es.r#loop = es.r#loop.wrapping_add(1);
        format_log1(
            ft,
            es,
            c"format_expand1",
            c"expanding format: %s",
            fmt_args![fmt],
        );
        let expanded = if es.flags & FORMAT_EXPAND_TIME != 0 && fmt.to_bytes().contains(&b'%') {
            if es.time == 0 {
                es.time = time(core::ptr::null_mut());
                es.tm = tm::local(es.time).unwrap_or_default();
            }
            let Some(text) = format_strftime(8192, fmt, &es.tm) else {
                format_log1(
                    ft,
                    es,
                    c"format_expand1",
                    c"format is too long",
                    fmt_args![],
                );
                return CString::default();
            };
            if format_logging(ft) != 0 && text.as_c_str() != fmt {
                format_log1(
                    ft,
                    es,
                    c"format_expand1",
                    c"after time expanded: %s",
                    fmt_args![text.as_c_str()],
                );
            }
            Some(text)
        } else {
            None
        };
        let fmt = expanded.as_deref().unwrap_or(fmt);
        let bytes = fmt.to_bytes();
        let mut position = 0;
        let mut style_end = None;
        while position < bytes.len() {
            if bytes[position] != b'#' {
                buf.push(bytes[position]);
                position += 1;
                continue;
            }
            let hash = position;
            position += 1;
            let Some(&ch) = bytes.get(position) else {
                break;
            };
            position += 1;
            match ch {
                b'(' => {
                    let mut brackets = 1;
                    let mut end = position;
                    while end < bytes.len() {
                        if bytes[end] == b'(' {
                            brackets += 1;
                        }
                        if bytes[end] == b')' {
                            brackets -= 1;
                            if brackets == 0 {
                                break;
                            }
                        }
                        end += 1;
                    }
                    if bytes.get(end) != Some(&b')') || brackets != 0 {
                        break;
                    }
                    let name =
                        CString::new(&bytes[position..end]).expect("format job name has no NUL");
                    format_log1(
                        ft,
                        es,
                        c"format_expand1",
                        c"found #(): %s",
                        fmt_args![name.as_c_str()],
                    );
                    let out =
                        if ft.flags & FORMAT_NOJOBS != 0 || es.flags & FORMAT_EXPAND_NOJOBS != 0 {
                            format_log1(ft, es, c"format_expand1", c"#() is disabled", fmt_args![]);
                            CString::default()
                        } else {
                            let out = format_job_get(ft, es, &name);
                            format_log1(
                                ft,
                                es,
                                c"format_expand1",
                                c"#() result: %s",
                                fmt_args![out.as_c_str()],
                            );
                            out
                        };
                    buf.extend_from_slice(out.as_bytes());
                    position = end + 1;
                    continue;
                }
                b'{' => {
                    let suffix = CStr::from_bytes_with_nul(&fmt.to_bytes_with_nul()[hash..])
                        .expect("format suffix retains the input terminator");
                    let Some(offset) = format_skip1(Some((ft, es)), suffix, c"}") else {
                        break;
                    };
                    let end = hash + offset;
                    let key = &bytes[position..end];
                    format_log1(
                        ft,
                        es,
                        c"format_expand1",
                        c"found #{}: %.*s",
                        fmt_args![key.len() as core::ffi::c_int, key],
                    );
                    if format_replace(ft, es, key, &mut buf) != 0 {
                        break;
                    }
                    position = end + 1;
                    continue;
                }
                b'[' | b'#' => {
                    let mut end = position - usize::from(ch == b'[');
                    while bytes.get(end) == Some(&b'#') {
                        end += 1;
                    }
                    if bytes.get(end) == Some(&b'[') {
                        let suffix = CStr::from_bytes_with_nul(&fmt.to_bytes_with_nul()[hash..])
                            .expect("format style suffix retains the input terminator");
                        style_end =
                            format_skip1(Some((ft, es)), suffix, c"]").map(|offset| hash + offset);
                        format_log1(
                            ft,
                            es,
                            c"format_expand1",
                            c"found #*%zu[",
                            fmt_args![end - hash],
                        );
                        buf.extend_from_slice(&bytes[hash..=end]);
                        position = end + 1;
                        continue;
                    }
                }
                b'}' | b',' => {}
                _ => {
                    let named = if style_end.is_none_or(|end| position > end) {
                        if ch.is_ascii_uppercase() {
                            format_upper[(ch - b'A') as usize]
                        } else if ch.is_ascii_lowercase() {
                            format_lower[(ch - b'a') as usize]
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let Some(named) = named else {
                        buf.push(b'#');
                        buf.push(ch);
                        continue;
                    };
                    format_log1(
                        ft,
                        es,
                        c"format_expand1",
                        c"found #%c: %s",
                        fmt_args![ch as core::ffi::c_int, named],
                    );
                    if format_replace(ft, es, named.to_bytes(), &mut buf) != 0 {
                        break;
                    }
                    continue;
                }
            }
            format_log1(
                ft,
                es,
                c"format_expand1",
                c"found #%c",
                fmt_args![ch as core::ffi::c_int],
            );
            buf.push(ch);
        }
        let result = CString::new(buf).expect("expanded format has no NUL");
        format_log1(
            ft,
            es,
            c"format_expand1",
            c"result is: %s",
            fmt_args![result.as_c_str()],
        );
        es.r#loop = es.r#loop.wrapping_sub(1);
        result
    }
}
/// Expands time directives and formats into an owned result.
///
/// Starts fresh expansion state with `strftime` processing enabled before normal
/// substitution. Local time is sampled when time processing is first needed;
/// `strftime` failure produces empty text. Other result, callback and job-cache
/// effects are those of [`format_expand`], including jobs that may outlive this
/// call. There is no separate error result.
///
/// # Safety
/// Meet [`format_expand`]'s server-thread, access and synchronous pane-liveness
/// requirements throughout this call, including nested expansion and callbacks.
pub unsafe fn format_expand_time(ft: &mut format_tree, fmt: &CStr) -> CString {
    unsafe {
        let mut es = format_expand_state {
            flags: FORMAT_EXPAND_TIME,
            start_time: get_timer(),
            ..Default::default()
        };
        format_expand1(ft, &mut es, fmt)
    }
}
/// Expands `fmt` against `ft`, returning owned text without a separate error result.
///
/// Starts fresh recursion/time-budget state, using the context's existing weak
/// entity observations and retained job client. Observed entities may have expired
/// since defaults were created; lookups retain their existing absent behavior.
/// Syntax errors and expansion limits can produce empty or partial output.
///
/// Lookups may synchronously call builtin/custom callbacks and plugin resolvers;
/// custom values may be cached in `ft`. Loops create temporary contexts with fresh
/// defaults, including pane-mode callbacks. `#()` may start or replace shell jobs
/// and mutate the retained client's job cache, or the global cache when no job
/// client was supplied. It uses currently cached output without waiting for job
/// completion. Later job callbacks update caches and may request status redraws.
/// Verbose formatting may also print diagnostics through the observed queue item.
///
/// # Safety
/// Run on the initialized server thread. Exclude conflicting entity, queue,
/// format-entry and job-cache borrows across this call, including access through
/// synchronous callbacks and nested formats. Keep resolved pane allocations alive
/// while their payloads are used; this does not require weak targets to remain
/// alive between context creation and expansion.
pub unsafe fn format_expand(ft: &mut format_tree, fmt: &CStr) -> CString {
    unsafe {
        let mut es = format_expand_state {
            start_time: get_timer(),
            ..Default::default()
        };
        format_expand1(ft, &mut es, fmt)
    }
}
pub unsafe fn format_single(
    item: Option<&cmdq_item>,
    fmt: &CStr,
    c: Option<&client>,
    s: Option<&session>,
    wl: Option<&winlink>,
    wp: Option<&impl crate::WindowPane>,
) -> CString {
    unsafe {
        let mut ft = format_create_defaults(item, c, s, wl, wp);
        format_expand(&mut ft, fmt)
    }
}
/// Expands `fmt` against the state `fs` resolved to, giving back the answer
/// the caller owns.
pub unsafe fn format_single_from_state(
    item: Option<&cmdq_item>,
    fmt: &CStr,
    c: Option<&client>,
    fs: &cmd_find_state,
) -> CString {
    unsafe {
        let mut ft = format_create_from_state(item, c, fs);
        format_expand(&mut ft, fmt)
    }
}
/// Creates a fresh target context, expands `fmt`, then drops that local context.
///
/// Uses [`format_create_from_target`]'s independent job/drawn clients and existing
/// target inheritance, followed immediately by [`format_expand`]. The returned
/// text is owned. Format jobs live in client/global caches and may outlive the
/// local context; dropping it does not wait for their completion.
///
/// # Safety
/// Meet both operations' server-thread, access and synchronous pane-liveness
/// requirements. Hold no entity, queue or cache borrow that conflicts with
/// defaults, expansion or their callbacks across this call.
pub unsafe fn format_single_from_target(item: &cmdq_item, fmt: &CStr) -> CString {
    unsafe {
        let mut ft = format_create_from_target(item);
        format_expand(&mut ft, fmt)
    }
}
pub unsafe fn format_create_defaults(
    item: Option<&cmdq_item>,
    c: Option<&client>,
    s: Option<&session>,
    wl: Option<&winlink>,
    wp: Option<&impl crate::WindowPane>,
) -> Box<format_tree> {
    unsafe {
        format_create_defaults_for_client(item, c.and_then(client_ref_of).as_ref(), s, wl, wp)
    }
}
unsafe fn format_create_defaults_for_client(
    item: Option<&cmdq_item>,
    c: Option<&ClientRef>,
    s: Option<&session>,
    wl: Option<&winlink>,
    wp: Option<&impl crate::WindowPane>,
) -> Box<format_tree> {
    unsafe {
        let cmdq_client = item.and_then(cmdq_item::client);
        let mut ft = format_create_for_client(
            cmdq_client.as_ref(),
            item,
            FORMAT_NONE,
            0 as core::ffi::c_int,
        );
        format_defaults_for_client(&mut ft, c, s, wl, wp);
        ft
    }
}
pub unsafe fn format_create_from_state(
    item: Option<&cmdq_item>,
    c: Option<&client>,
    fs: &cmd_find_state,
) -> Box<format_tree> {
    unsafe { format_create_from_state_for_client(item, c.and_then(client_ref_of).as_ref(), fs) }
}
/// Creates a context from command target observations and a drawn client.
///
/// The item's current optional client supplies format jobs independently of `c`;
/// queue formats and mouse state are copied before defaults. An absent drawn
/// client stays absent, and retained clients need not be registered. State session
/// and pane observations resolve their original allocations while alive; links
/// resolve their owning session/index, including replacements at that index.
/// No registration or pane-membership checks are added.
///
/// Resolved explicit pane/link/session targets determine context type before
/// inheritance. An absent session inherits the drawn client's attached session;
/// an absent link inherits the resulting session's current link; an absent pane
/// inherits the resolved link's active pane. Explicit links retain window
/// precedence over moved panes. Entity observations remain weak. Defaults run
/// synchronous mode formats and select the current paste buffer; expansion is
/// separate.
///
/// # Safety
/// Run on the initialized server thread, preventing pane destruction and mutable
/// access to explicit or inherited clients, sessions, links, panes and modes
/// during this call. Mode formats read entities and write only the format tree.
/// No entity payload borrow escapes the call.
pub(crate) unsafe fn format_create_from_state_for_client(
    item: Option<&cmdq_item>,
    c: Option<&ClientRef>,
    fs: &cmd_find_state,
) -> Box<format_tree> {
    unsafe {
        let pane = fs.pane_ref();
        let winlink = fs.winlink_ref();
        format_create_defaults_for_client(
            item,
            c,
            fs.session()
                .as_ref()
                .map(|reference| reference.as_session()),
            winlink.as_ref().and_then(WinlinkRef::get),
            pane.as_ref().and_then(|pane| pane.get()),
        )
    }
}
/// Creates a context from the item's current target client and target state.
///
/// The item's current optional client is retained for jobs and working-directory
/// lookup, independently of the target client used for drawn-client fields. An
/// absent client in either role stays absent. Queue formats and mouse state are
/// copied at creation; the item itself is observed weakly.
///
/// Target session/pane observations resolve their original live allocations;
/// links resolve their owning session/index, including replacements at that
/// index. Retained clients/sessions need not be registered, and pane registration
/// or membership checks are not added. Resolved explicit pane/link/session targets
/// determine context type before inheritance. An absent session inherits the drawn
/// client's attached session; an absent link inherits the resulting session's
/// current link; an absent pane inherits the resolved link's active pane. An
/// explicit link keeps window precedence over a moved pane.
///
/// Defaults synchronously run pane-mode formats and select the current paste
/// buffer. Drawn client, session, link, window and pane observations remain weak;
/// they need not stay alive until a later expansion. String expansion is separate.
///
/// # Safety
/// Run on the initialized server thread. Exclude conflicting client, session,
/// link, pane, mode and queue access during construction and its mode callbacks,
/// keeping resolved panes alive while their payloads are used. No entity payload
/// borrow escapes the call. Later expansion has its own [`format_expand`] access
/// requirements.
pub unsafe fn format_create_from_target(item: &cmdq_item) -> Box<format_tree> {
    unsafe {
        let tc = item.target_client();
        format_create_from_state_for_client(Some(item), tc.as_ref(), &item.target)
    }
}
/// Adds session defaults using the retained session's current link and active pane.
///
/// The session need not be registered. Link identity includes its owning session
/// and index, even when a window is linked more than once. Missing links or panes
/// supply no defaults for that entity. The context observes entities weakly using
/// the existing format-tree rules; this does not extend their allocation lifetime.
/// No drawn client is supplied or inferred from the context's job client.
///
/// Defaults include synchronous pane-mode formats and the current paste buffer.
/// Existing entries remain available to mode formats; string expansion happens
/// only when requested separately. No commands, hooks or notifications are run.
///
/// # Safety
/// Run on the initialized server thread, excluding mutable access to the session
/// and its current link, active pane and mode during this call. The mode formats
/// callback reads these entities and writes only the format tree. No payload
/// borrow escapes the call; callers may then expand or replace the context.
pub(crate) unsafe fn format_defaults_for_session(ft: &mut format_tree, s: &SessionRef) {
    unsafe {
        format_defaults_for_client(
            ft,
            None,
            Some(s.as_session()),
            None,
            None::<&crate::types::window_pane>,
        );
    }
}

/// Adds defaults for the link currently at a retained session/index pair.
///
/// Returns false without changing the context if the index is no longer linked;
/// it never falls back to the session's current link. The session need not be
/// registered. A replacement at the same index is resolved at call time. The
/// explicit link supplies window identity and its inherited active pane, keeping
/// the context's type as window even when pane defaults are available. No drawn
/// client is supplied or inferred from the context's job client.
///
/// Uses the existing weak format-tree observations without extending entity
/// lifetimes. Existing entries are available to synchronous pane-mode formats;
/// the current paste buffer is selected before returning. String expansion is
/// separate. No commands, hooks or notifications are run.
///
/// # Safety
/// Run on the initialized server thread, excluding mutable access to the owning
/// session, resolved link, active pane and mode during this call. The mode formats
/// callback reads these entities and writes only the format tree. No payload
/// borrow escapes the call; callers may then expand or replace the context.
pub(crate) unsafe fn format_defaults_for_link(
    ft: &mut format_tree,
    link: &crate::window::WinlinkRef,
) -> bool {
    let Some(wl) = link.get() else {
        return false;
    };
    unsafe {
        format_defaults_for_client(
            ft,
            None,
            Some(link.session().as_session()),
            Some(wl),
            None::<&crate::types::window_pane>,
        );
    }
    true
}

/// Adds defaults for a session/index link and an explicitly observed pane.
///
/// Returns false without changing the context if the pane allocation is gone or
/// the index is no longer linked. The retained session need not be registered;
/// a replacement link at the same index is resolved at call time. Pane identity
/// is independent of membership: no registration or owning-window check is added.
/// The explicit pane selects pane context type and never inherits the active pane.
/// The link supplies the owning session/index and takes precedence for window
/// defaults, including when the pane has moved. No drawn client is inferred.
///
/// Uses existing weak format observations without extending entity lifetimes.
/// Existing entries are available to synchronous pane-mode formats, followed by
/// current paste-buffer selection. Expansion is separate; no commands, hooks or
/// notifications run. Missing targets run no callbacks.
///
/// # Safety
/// Run on the initialized server thread, preventing pane destruction and mutable
/// access to the session, resolved link, pane and its mode during this call.
/// The mode formats callback reads these entities and writes only the format
/// tree. No payload borrow escapes the call.
pub(crate) unsafe fn format_defaults_for_link_pane(
    ft: &mut format_tree,
    link: &crate::window::WinlinkRef,
    pane: &RustWindowPaneWeak,
) -> bool {
    unsafe {
        let Some(wp) = pane.get() else {
            return false;
        };
        let Some(wl) = link.get() else {
            return false;
        };
        format_defaults_for_client(
            ft,
            None,
            Some(link.session().as_session()),
            Some(wl),
            Some(wp),
        );
    }
    true
}

/// Adds defaults from independently optional entity handles.
///
/// The retained client supplies drawn-client fields, independently of the format
/// job client; an absent drawn client stays absent. Retained clients and sessions
/// need not be registered. Links resolve their owning session/index at call time,
/// including a replacement at that index; panes resolve their observed allocation
/// without registration or membership checks. A missing link or expired pane is
/// treated as an absent argument.
///
/// The context type uses the resolved explicit pane, link or session before
/// inheritance. An absent session inherits the drawn client's attached session;
/// an absent link inherits that session's current link; an absent pane inherits
/// the resolved link's active pane. Explicit links preserve their own session/index
/// identity and take precedence over panes for window defaults. A moved pane is
/// not replaced merely because it no longer belongs to the supplied link.
///
/// Existing entries are available to synchronous pane-mode formats, followed by
/// current paste-buffer selection. Expansion and enumeration are separate. No
/// commands, hooks or notifications run. Entity observations remain weak and do
/// not extend allocation lifetimes.
///
/// # Safety
/// Run on the initialized server thread, preventing destruction of resolved panes
/// and mutable access to explicit or inherited clients, sessions, links, panes and
/// modes during the call. Mode formats read those entities and write only the
/// format tree. No payload borrow escapes the call.
pub(crate) unsafe fn format_defaults_for_handles(
    ft: &mut format_tree,
    c: Option<&ClientRef>,
    s: Option<&SessionRef>,
    link: Option<&WinlinkRef>,
    pane: Option<&RustWindowPaneWeak>,
) {
    unsafe {
        format_defaults_for_client(
            ft,
            c,
            s.map(|s| s.as_session()),
            link.and_then(WinlinkRef::get),
            pane.and_then(|pane| pane.get()),
        );
    }
}

pub unsafe fn format_defaults(
    ft: &mut format_tree,
    c: Option<&client>,
    s: Option<&session>,
    wl: Option<&winlink>,
    wp: Option<&impl crate::WindowPane>,
) {
    unsafe { format_defaults_for_client(ft, c.and_then(client_ref_of).as_ref(), s, wl, wp) }
}
unsafe fn format_defaults_for_client(
    ft: &mut format_tree,
    c: Option<&ClientRef>,
    s: Option<&session>,
    wl: Option<&winlink>,
    wp: Option<&impl crate::WindowPane>,
) {
    unsafe {
        match c.filter(|c| c.name().is_some()) {
            Some(c) => log_debug(c"%s: c=%s", fmt_args![c"format_defaults", c.name()]),
            None => log_debug(c"%s: c=none", fmt_args![c"format_defaults"]),
        }
        match s {
            Some(s) => log_debug(
                c"%s: s=$%u",
                fmt_args![c"format_defaults", crate::SessionIdentity::session_id(s)],
            ),
            None => log_debug(c"%s: s=none", fmt_args![c"format_defaults"]),
        }
        match wl {
            Some(wl) => log_debug(c"%s: wl=%u", fmt_args![c"format_defaults", wl.idx]),
            None => log_debug(c"%s: wl=none", fmt_args![c"format_defaults"]),
        }
        match wp {
            Some(wp) => log_debug(c"%s: wp=%%%u", fmt_args![c"format_defaults", wp.pane_id()]),
            None => log_debug(c"%s: wp=none", fmt_args![c"format_defaults"]),
        }
        if let (Some(c), Some(s)) = (c, s)
            && c.attached_session()
                .is_none_or(|attached| !attached.points_to(s))
        {
            log_debug(c"%s: session does not match", fmt_args![c"format_defaults"]);
        }
        if wp.is_some() {
            ft.type_0 = FORMAT_TYPE_PANE;
        } else if wl.is_some() {
            ft.type_0 = FORMAT_TYPE_WINDOW;
        } else if s.is_some() {
            ft.type_0 = FORMAT_TYPE_SESSION;
        } else {
            ft.type_0 = FORMAT_TYPE_UNKNOWN;
        }
        let attached = c.and_then(|c| c.attached_session());
        let s = s.or_else(|| attached.as_ref().map(|session| session.as_session()));
        let wl = wl.or_else(|| s.and_then(session::curw));
        let active_pane = if wp.is_none() {
            wl.and_then(|wl| {
                let window = wl.window_handle()?.clone();
                let id = window.active_pane_id()?;
                window.pane_by_id(id)
            })
        } else {
            None
        };
        if let Some(c) = c {
            format_defaults_client(ft, c);
        }
        if let Some(s) = s {
            format_defaults_session(ft, s);
        }
        if let Some(wl) = wl {
            format_defaults_winlink(ft, wl);
        }
        if let Some(wp) = wp {
            format_defaults_pane(ft, wp);
        } else if let Some(wp) = active_pane.as_ref().and_then(|pane| pane.get()) {
            format_defaults_pane(ft, wp);
        }
        if let Some(name) =
            with_paste_buffers(|buffers| buffers.top().map(|buffer| buffer.name.to_owned()))
        {
            format_defaults_paste_buffer(ft, name.as_c_str());
        }
    }
}
fn format_defaults_session(ft: &mut format_tree, s: &session) {
    ft.set_session(Some(s));
}
fn format_defaults_client(ft: &mut format_tree, c: &ClientRef) {
    if ft.session().is_none() {
        ft.s = c.attached_session().map(|session| session.downgrade());
    }
    ft.set_drawn_client(Some(c));
}
pub fn format_defaults_window(ft: &mut format_tree, w: &WindowRef) {
    ft.set_window(Some(w));
}
fn format_defaults_winlink(ft: &mut format_tree, wl: &winlink) {
    if ft.window().is_none()
        && let Some(window) = wl.window_handle()
    {
        format_defaults_window(ft, window);
    }
    ft.set_winlink(Some(wl));
}
pub unsafe fn format_defaults_pane(ft: &mut format_tree, wp: &impl crate::WindowPane) {
    unsafe {
        let window = wp.window_context();
        if ft.window().is_none()
            && let Some(window) = window.as_ref()
        {
            format_defaults_window(ft, window);
        }
        ft.set_pane(Some(wp));
        if let Some(wme) = window_pane_current_mode(wp) {
            wme.mode().formats(wme, ft);
        }
    }
}
pub fn format_defaults_paste_buffer(ft: &mut format_tree, name: &CStr) {
    ft.set_buffer(Some(name));
}
fn format_is_word_separator(ws: &CStr, gc: &grid_cell) -> core::ffi::c_int {
    if utf8_cstrhas(ws, &gc.data) != 0 {
        return 1 as core::ffi::c_int;
    }
    if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return 1 as core::ffi::c_int;
    }
    (gc.data.size as core::ffi::c_int == 1 as core::ffi::c_int && gc.data.data[0] == b' ')
        as core::ffi::c_int
}
pub unsafe fn format_grid_word(gd: &grid, mut x: u_int, mut y: u_int) -> Option<CString> {
    {
        let mut gl: Option<&grid_line>;
        let mut gc;
        let mut ud: Vec<utf8_data> = Vec::new();
        let mut end: u_int;
        let mut found: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut s: Option<CString> = None;
        let ws = (global_s_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .string_ref(c"word-separators");
        loop {
            gc = gd.cell(x, y);
            if !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0
                && format_is_word_separator(&ws, &gc) != 0
            {
                found = 1 as core::ffi::c_int;
                break;
            } else {
                if x == 0 as u_int {
                    if y == 0 as u_int {
                        break;
                    }
                    gl = grid_peek_line(gd, y.wrapping_sub(1 as u_int));
                    if !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                        break;
                    }
                    y = y.wrapping_sub(1);
                    x = gd.line_length(y);
                    if x == 0 as u_int {
                        break;
                    }
                }
                x = x.wrapping_sub(1);
            }
        }
        loop {
            if found != 0 {
                end = gd.line_length(y);
                if end == 0 as u_int || x == end.wrapping_sub(1 as u_int) {
                    if y == gd.hsize.wrapping_add(gd.sy).wrapping_sub(1 as u_int) {
                        break;
                    }
                    gl = grid_peek_line(gd, y);
                    if !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                        break;
                    }
                    y = y.wrapping_add(1);
                    x = 0 as u_int;
                } else {
                    x = x.wrapping_add(1);
                }
            }
            found = 1 as core::ffi::c_int;
            gc = gd.cell(x, y);
            if gc.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0 {
                continue;
            }
            if format_is_word_separator(&ws, &gc) != 0 {
                break;
            }
            ud.push(gc.data);
        }
        if !ud.is_empty() {
            s = Some(utf8_vec_tocstr(&ud));
        }
        s
    }
}
pub fn format_grid_line(gd: &grid, y: u_int) -> CString {
    let mut gc;
    let mut ud: Vec<utf8_data> = Vec::new();
    let mut x: u_int = 0;
    while x < gd.line_length(y) {
        gc = gd.cell(x, y);
        if !(gc.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0) {
            ud.push(gc.data);
            if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
                utf8_set(&mut *ud.last_mut().unwrap(), '\t' as i32 as u_char);
            }
        }
        x = x.wrapping_add(1);
    }
    utf8_vec_tocstr(&ud)
}
pub fn format_grid_hyperlink(gd: &grid, mut x: u_int, y: u_int, s: &RustScreen) -> Option<CString> {
    let mut gc;
    loop {
        gc = gd.cell(x, y);
        if !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        if x == 0 as u_int {
            return None;
        }
        x = x.wrapping_sub(1);
    }
    let hyperlinks = s.hyperlinks();
    if gc.link == 0 as u_int {
        return None;
    }
    let (uri, _, _) = hyperlinks.get(gc.link)?;
    Some(uri.to_owned())
}
use crate::screen::RustScreen;

#[cfg(test)]
#[path = "../tests/test_format_expand_focused.rs"]
mod focused_tests;
