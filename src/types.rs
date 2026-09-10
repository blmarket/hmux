//! Type declarations shared by every generated module.
//!
//! `c2rust` emitted these once per translation unit, which made each
//! module's `grid_cell` a distinct Rust type.  They are collected here
//! so that a name means one type crate-wide; every module glob-imports
//! this module, and the few modules that hold a private definition of a
//! type that is opaque here keep their own copy, which shadows the glob.

use crate::args::RustArguments;
use crate::cmd::{
    CmdListRef, DisplayPanesRef, SourceFileRef, cmd_command_prompt_cdata, cmd_confirm_before_data,
    cmd_load_buffer_data,
};
use crate::screen::Screen;
use ::core::cell::UnsafeCell;
use ::std::cell::RefCell;
use ::std::fmt;
use ::std::rc::{Rc, Weak};

unsafe extern "C" {
    pub type dirent;
    pub type term;
}

pub use crate::reactor::{ByteBuffer, IoHandle, SignalHandle, Stream, TimerHandle};
pub type TERMINAL = term;
pub use crate::args::args;
pub use crate::args::args_command_state;
pub use crate::cmd::{CmdqListRef, cmdq_list};
pub use crate::compat::ibufqueue;
pub use crate::compat::msgbuf;
pub use crate::control::control_state;
use crate::environ::RustEnvironment;
pub use crate::format::format_job;
pub use crate::format::format_job_tree;
pub use crate::format::format_tree;
pub use crate::input::{InputCtxRef, input_ctx};
pub use crate::input::{input_request, input_request_handle};
pub use crate::job::{JobEvent, job};
pub use crate::key_bindings::{key_binding, key_bindings, key_table};
pub use crate::modes::mode_tree_data;
pub use crate::modes::mode_tree_item;
pub use crate::modes::mode_tree_menu;
use crate::modes::window_buffer_modedata;
use crate::modes::window_clock_mode_data;
use crate::modes::window_copy_mode_data;
use crate::modes::{window_client_itemdata, window_client_modedata};
pub use crate::modes::{window_customize_itemdata, window_customize_modedata};
pub use crate::modes::{window_tree_itemdata, window_tree_modedata};
pub use crate::options::options_array_item_t;
pub use crate::options::options_entry;
pub use crate::options::{OptionsRef, RustOptionsRef};
pub use crate::overlay::{MenuDataRef, menu_data};
pub use crate::overlay::{PopupDataRef, PopupDataWeak, popup_data};
pub use crate::proc::tmuxpeer;
pub use crate::proc::{PeerRef, ProcessRef, tmuxproc};
use crate::prompt_history::PromptHistoryType;
pub use crate::screen::screen_sel;
pub use crate::screen::screen_titles;
pub use crate::screen::screen_write_citem;
pub use crate::screen::screen_write_cline;
pub use crate::session::session;
pub use crate::session::{session_group, session_groups_t};
pub use crate::status::status_prompt_menu;
use crate::terminfo::TtyCode;
pub use crate::tty::tty_key;
pub use crate::window::window_pane_input_data;
pub use ::libc::{FILE, sockaddr_storage, termios, utsname};
pub type __off64_t = core::ffi::c_long;
pub type __off_t = core::ffi::c_long;
pub type __uint64_t = u64;
pub type __blkcnt_t = core::ffi::c_long;
pub type __blksize_t = core::ffi::c_long;
pub(crate) type window_mode_init = unsafe fn(
    &mut window_mode_entry,
    crate::window::RustWindowPaneWeak,
    Option<&cmd_find_state>,
    Option<&RustArguments>,
);
pub type window_mode_command = unsafe fn(
    &mut window_mode_entry,
    Option<&mut client>,
    Option<&session>,
    Option<&winlink>,
    &RustArguments,
    Option<&mut mouse_event>,
) -> ();
pub type __clock_t = core::ffi::c_long;
pub type __clockid_t = core::ffi::c_int;
pub type __dev_t = core::ffi::c_ulong;
pub type __gid_t = core::ffi::c_uint;
pub type __ino_t = core::ffi::c_ulong;
pub type __int32_t = i32;
pub type __mode_t = core::ffi::c_uint;
pub type __nlink_t = core::ffi::c_ulong;
pub type __pid_t = core::ffi::c_int;
pub type __re_long_size_t = core::ffi::c_ulong;
pub type __sighandler_t = Option<unsafe extern "C" fn(core::ffi::c_int) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sigset_t {
    pub __val: [core::ffi::c_ulong; 16],
}
pub type __size_t = usize;
pub type __socket_type = core::ffi::c_uint;
pub type __socklen_t = core::ffi::c_uint;
pub type __suseconds_t = core::ffi::c_long;
pub type __syscall_slong_t = core::ffi::c_long;
pub type __time_t = core::ffi::c_long;
pub type __u_char = core::ffi::c_uchar;
pub type __u_int = core::ffi::c_uint;
pub type __u_short = core::ffi::c_ushort;
pub type __uid_t = core::ffi::c_uint;
pub type __uint16_t = u16;
pub type __uint32_t = u32;
pub type __uint8_t = u8;
pub type __useconds_t = core::ffi::c_uint;
pub type args_parse_type = core::ffi::c_uint;
pub type u_int = __u_int;
pub type args_parse_cb =
    Option<unsafe fn(&args, u_int, &mut Option<std::ffi::CString>) -> args_parse_type>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse_t {
    pub template: &'static core::ffi::CStr,
    pub lower: core::ffi::c_int,
    pub upper: core::ffi::c_int,
    pub cb: args_parse_cb,
}
impl args_parse_t {
    /// Builds an argument parser specification usable in static command entries.
    pub const fn new(
        template: &'static core::ffi::CStr,
        lower: core::ffi::c_int,
        upper: core::ffi::c_int,
        cb: args_parse_cb,
    ) -> Self {
        Self {
            template,
            lower,
            upper,
            cb,
        }
    }
}
pub type bitstr_t = core::ffi::c_uchar;
pub type box_lines = core::ffi::c_int;
pub type cc_t = core::ffi::c_uchar;
pub type client_theme = core::ffi::c_uint;
pub type clockid_t = __clockid_t;
/// The state a pane's input read carries, shared the same way.
#[derive(Clone)]
pub struct PaneInputRef(Rc<RefCell<window_pane_input_data>>);

impl PaneInputRef {
    pub(crate) fn new(value: window_pane_input_data) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn with<R>(&self, operation: impl FnOnce(&window_pane_input_data) -> R) -> R {
        operation(&self.0.borrow())
    }

    pub(crate) fn with_mut<R>(
        &self,
        operation: impl FnOnce(&mut window_pane_input_data) -> R,
    ) -> R {
        operation(&mut self.0.borrow_mut())
    }
}

impl fmt::Debug for PaneInputRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("PaneInputRef")
            .field(&Rc::as_ptr(&self.0))
            .finish()
    }
}

/// A screen read that keeps a shared screen's borrow checked until it ends.
pub enum ScreenBorrow<'a> {
    Owned(&'a RustScreen),
    Shared(std::cell::Ref<'a, RustScreen>),
}

impl core::ops::Deref for ScreenBorrow<'_> {
    type Target = RustScreen;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(screen) => screen,
            Self::Shared(screen) => screen,
        }
    }
}

/// A strong owner of a screen with checked shared and exclusive borrows.
#[derive(Clone)]
pub(crate) struct ScreenRef(Rc<SharedScreen>);

struct SharedScreen {
    screen: RefCell<RustScreen>,
    writing: std::cell::Cell<bool>,
}

pub(crate) struct ScreenWriteLease<'a> {
    target: &'a ScreenRef,
}

impl<'a> ScreenWriteLease<'a> {
    pub(crate) fn borrow(&self) -> std::cell::Ref<'a, RustScreen> {
        self.target.0.screen.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'a, RustScreen> {
        self.target.0.screen.borrow_mut()
    }
}

impl Drop for ScreenWriteLease<'_> {
    fn drop(&mut self) {
        self.target.0.writing.set(false);
    }
}

impl Default for ScreenRef {
    fn default() -> Self {
        Self::new(RustScreen::default())
    }
}

impl ScreenRef {
    pub(crate) fn new(value: RustScreen) -> Self {
        Self(Rc::new(SharedScreen {
            screen: RefCell::new(value),
            writing: std::cell::Cell::new(false),
        }))
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, RustScreen> {
        self.0.screen.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, RustScreen> {
        assert!(!self.0.writing.get(), "screen has an active writer");
        self.0.screen.borrow_mut()
    }

    pub(crate) fn begin_write(&self) -> ScreenWriteLease<'_> {
        assert!(!self.0.writing.replace(true), "screen has an active writer");
        ScreenWriteLease { target: self }
    }

    pub(crate) fn is_unique(&self) -> bool {
        Rc::strong_count(&self.0) == 1
    }

    #[cfg(test)]
    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Makes a non-owning observation of this screen.
    pub(crate) fn downgrade(&self) -> ScreenWeak {
        ScreenWeak(Rc::downgrade(&self.0))
    }
}

/// A non-owning observation of a screen. The status line watches the
/// overlay it is drawing on this way, so that the message and prompt slots
/// alone decide how long the screen lives.
#[derive(Clone)]
pub(crate) struct ScreenWeak(Weak<SharedScreen>);

impl ScreenWeak {
    /// The screen if the slots that held it still do, as the handle that
    /// keeps it alive rather than a pointer into one that has gone.
    pub(crate) fn upgrade(&self) -> Option<ScreenRef> {
        self.0.upgrade().map(ScreenRef)
    }
}

/// A strong owner of a window allocation. The raw pointer returned by
/// [`as_ptr`](Self::as_ptr) is a borrowed compatibility view; an owning edge
/// must clone or drop this handle instead of retaining that pointer. Winlinks,
/// queued alerts and notifications use strong handles, while the id registry
/// and timer callbacks use [`WindowWeak`]. Windows hold sole ownership of their panes;
/// each pane observes its window weakly. Pane teardown clears the translated
/// window pointer before outstanding pane references can outlive the window.
///
/// Payload borrows are checked across cloned handles and remain checked until
/// their guards are dropped. Ownership does not grant implicit payload references.
///
/// ```compile_fail
/// use tmux_c2rs::types::{WindowRef, window};
/// fn shared(owner: &WindowRef) -> &window { owner }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::types::{WindowRef, window};
/// fn exclusive(owner: &mut WindowRef) -> &mut window { owner }
/// ```
#[derive(Clone)]
pub struct WindowRef(Rc<RefCell<WindowStorage>>);

#[derive(Clone)]
pub(crate) struct WindowWeak(Weak<RefCell<WindowStorage>>);

pub(crate) struct WindowStorage {
    pub(crate) value: window,
    pub(crate) id_registration: Option<crate::handle_registry::HandleRegistration<WindowWeak>>,
}

impl WindowRef {
    /// Copies the name so the result does not retain a payload borrow.
    pub(crate) fn window_name(&self) -> Option<std::ffi::CString> {
        crate::WindowNameState::window_name(&*self.as_window()).map(core::ffi::CStr::to_owned)
    }

    /// Snapshots pane membership without keeping panes alive or retaining a payload borrow.
    pub(crate) fn panes(&self) -> Vec<RustWindowPaneWeak> {
        self.as_window()
            .panes
            .iter()
            .map(RustWindowPaneRef::downgrade)
            .collect()
    }

    pub(crate) fn pane_count(&self) -> usize {
        self.as_window().panes.len()
    }

    pub(crate) fn window_id(&self) -> u32 {
        crate::Window::window_id(&*self.as_window())
    }

    pub(crate) fn dimensions(&self) -> crate::WindowDimensions {
        crate::WindowDimensionsState::dimensions(&*self.as_window())
    }

    pub(crate) fn timestamps(&self) -> crate::WindowTimestamps {
        crate::WindowTimestampState::timestamps(&*self.as_window())
    }

    pub(crate) fn previous_layout(&self) -> Option<core::ffi::c_int> {
        crate::WindowLayoutSelectionState::previous_layout(&*self.as_window())
    }

    pub(crate) fn scrollbar_settings(&self) -> crate::WindowScrollbarSettings {
        crate::WindowScrollbarState::scrollbar_settings(&*self.as_window())
    }

    pub(crate) fn set_window_name(&self, value: Option<&core::ffi::CStr>) {
        crate::WindowNameState::set_window_name(&mut *self.as_window_mut(), value);
    }

    pub(crate) fn remember_layout(&self, value: core::ffi::c_int) {
        crate::WindowLayoutSelectionState::remember_layout(&mut *self.as_window_mut(), value);
    }

    pub(crate) fn set_saved_layout(&self, value: Option<&core::ffi::CStr>) {
        crate::WindowSavedLayoutState::set_saved_layout(&mut *self.as_window_mut(), value);
    }

    pub(crate) fn set_activity_time(&self, value: timeval) {
        crate::WindowTimestampState::set_activity_time(&mut *self.as_window_mut(), value);
    }

    #[cfg(test)]
    pub(crate) fn set_name_update_time(&self, value: timeval) {
        crate::WindowTimestampState::set_name_update_time(&mut *self.as_window_mut(), value);
    }

    #[cfg(test)]
    pub(crate) fn set_scrollbar_settings(&self, value: crate::WindowScrollbarSettings) {
        crate::WindowScrollbarState::set_scrollbar_settings(&mut *self.as_window_mut(), value);
    }

    #[cfg(test)]
    pub(crate) fn set_pixels(&self, value: crate::WindowPixelSize) {
        crate::WindowDimensionsState::set_pixels(&mut *self.as_window_mut(), value);
    }

    pub(crate) fn active_pane(&self) -> Option<RustWindowPaneWeak> {
        crate::window::window_active_pane(&self.as_window())
    }

    pub(crate) fn active_pane_id(&self) -> Option<u_int> {
        self.active_pane().map(|pane| pane.id())
    }

    pub(crate) fn options(&self) -> RustOptionsRef {
        self.as_window().options_ref().clone()
    }

    pub(crate) fn winlinks(&self) -> impl Iterator<Item = crate::window::WinlinkRef> + use<> {
        crate::window::winlinks_into(&self.as_window())
    }

    pub(crate) fn alert_flags(&self) -> core::ffi::c_int {
        self.as_window().flags & crate::alerts::WINDOW_ALERTFLAGS
    }

    pub(crate) fn add_alert_flags(&self, flags: core::ffi::c_int) -> bool {
        let mut window = self.as_window_mut();
        let changed = window.flags & flags != flags;
        window.flags |= flags;
        changed
    }

    pub(crate) fn queue_alerts(&self) -> bool {
        crate::WindowAlertQueueState::queue_alerts(&mut *self.as_window_mut())
    }

    pub(crate) fn finish_alerts(&self) {
        let mut window = self.as_window_mut();
        crate::WindowAlertQueueState::clear_queued_alerts(&mut *window);
        window.flags &= !crate::alerts::WINDOW_ALERTFLAGS;
    }

    pub(crate) fn reset_alert_timer(&self, seconds: __time_t, callback: impl FnMut() + 'static) {
        use crate::reactor::Timer;

        let mut window = self.as_window_mut();
        if !window.alerts_timer.is_set() {
            window.alerts_timer.set_callback(callback);
        }
        window.flags &= !crate::alerts::WINDOW_SILENCE;
        window.alerts_timer.disarm();
        if seconds != 0 {
            window.alerts_timer.arm(timeval {
                tv_sec: seconds,
                tv_usec: 0,
            });
        }
    }

    /// Observes a registered pane whose recorded membership is this window.
    /// This lookup does not borrow the window payload or search its pane list;
    /// manually constructed panes must be registered and assigned membership.
    pub fn pane_by_id(&self, id: u32) -> Option<RustWindowPaneWeak> {
        let pane = crate::window::window_pane_find_by_id(id)?;
        pane.window().filter(|owner| owner.ptr_eq(self))?;
        Some(pane)
    }

    pub(crate) fn new(value: window) -> Self {
        let reference = Self(Rc::new(RefCell::new(WindowStorage {
            value,
            id_registration: None,
        })));
        reference.0.borrow_mut().value.owner = Some(reference.downgrade());
        reference
    }

    pub(crate) fn register_id(&self) {
        let registration = crate::window::register_window_id(self);
        self.0.borrow_mut().id_registration = Some(registration);
    }

    /// Borrows the window payload with runtime shared-borrow checking.
    pub(crate) fn as_window(&self) -> std::cell::Ref<'_, window> {
        std::cell::Ref::map(self.0.borrow(), |storage| &storage.value)
    }

    /// Borrows the window payload with runtime exclusive-borrow checking.
    pub(crate) fn as_window_mut(&self) -> std::cell::RefMut<'_, window> {
        std::cell::RefMut::map(self.0.borrow_mut(), |storage| &mut storage.value)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Returns the payload address without borrowing it. Dereferencing this
    /// pointer must respect all live guards and the allocation lifetime.
    pub(crate) fn as_ptr(&self) -> *mut window {
        unsafe { &raw mut (*self.0.as_ptr()).value }
    }

    pub(crate) fn downgrade(&self) -> WindowWeak {
        WindowWeak(Rc::downgrade(&self.0))
    }
}

impl WindowWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowRef> {
        self.0.upgrade().map(WindowRef)
    }
}

/// A strong owner of a session allocation. The raw pointer returned by
/// [`as_ptr`](Self::as_ptr) is a borrowed compatibility view; deferred jobs
/// and notifications own sessions by cloning this handle. The live-session
/// registry owns one handle while a session is discoverable, while clients
/// and target state remain observational back-pointers.
///
/// Ownership does not grant implicit shared or exclusive payload references.
///
/// ```compile_fail
/// use tmux_c2rs::types::{SessionRef, session};
/// fn shared(owner: &SessionRef) -> &session { owner }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::types::{SessionRef, session};
/// fn exclusive(owner: &mut SessionRef) -> &mut session { owner }
/// ```
#[derive(Clone)]
pub struct SessionRef(Rc<UnsafeCell<SessionStorage>>);

#[derive(Clone)]
pub(crate) struct SessionWeak(Weak<UnsafeCell<SessionStorage>>);

pub(crate) struct SessionStorage {
    pub(crate) value: session,
}

impl SessionRef {
    pub(crate) fn new(mut value: session) -> Self {
        Self(Rc::new_cyclic(|owner| {
            value.owner = Some(SessionWeak(owner.clone()));
            UnsafeCell::new(SessionStorage { value })
        }))
    }

    /// Returns a compatibility view of the session payload.
    ///
    /// # Safety
    /// The caller must prevent mutation for the lifetime of the returned
    /// reference, including mutation through other owners or reentrant callbacks.
    pub(crate) unsafe fn as_session(&self) -> &session {
        unsafe { &*self.as_ptr() }
    }

    /// Returns an exclusive compatibility view of the session payload.
    ///
    /// # Safety
    /// The caller must exclude all other access to the payload for this borrow's
    /// lifetime, including access through other owners or reentrant callbacks.
    pub(crate) unsafe fn as_session_mut(&mut self) -> &mut session {
        unsafe { &mut *self.as_ptr() }
    }

    /// Whether both handles own the same allocation.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn as_ptr(&self) -> *mut session {
        unsafe { &raw mut (*self.0.get()).value }
    }

    pub(crate) fn downgrade(&self) -> SessionWeak {
        SessionWeak(Rc::downgrade(&self.0))
    }

    pub(crate) fn points_to(&self, value: &session) -> bool {
        core::ptr::eq(self.as_ptr(), value)
    }
}

impl SessionWeak {
    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }

    pub(crate) fn upgrade(&self) -> Option<SessionRef> {
        self.0.upgrade().map(SessionRef)
    }
}

/// A strong owner of a live client allocation. Owning edges clone this handle
/// and observations use [`ClientWeak`]. Value snapshots use checked borrows;
/// compatibility field views require explicit unsafe borrows tied to the handle's lifetime.
/// The payload is stored in a [`RefCell`], but compatibility views use its raw
/// pointer to allow disjoint field borrows and do not perform borrow checks.
///
/// Ownership does not grant implicit shared or exclusive payload references.
///
/// ```compile_fail
/// use tmux_c2rs::types::{ClientRef, client};
/// fn shared(owner: &ClientRef) -> &client { owner }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::types::{ClientRef, client};
/// fn exclusive(owner: &mut ClientRef) -> &mut client { owner }
/// ```
#[derive(Clone)]
pub struct ClientRef(Rc<ClientStorage>);

#[derive(Clone)]
pub struct ClientWeak(Weak<ClientStorage>);

thread_local! {
    static NEXT_CLIENT_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub(crate) struct ClientStorage {
    id: u64,
    pub(crate) value: RefCell<client>,
}

impl ClientRef {
    /// Visits the client's existing format job cache.
    ///
    /// # Safety
    /// Exclude other cache access, including reentrant callbacks, during the visit.
    pub(crate) unsafe fn with_format_jobs<R>(
        &mut self,
        visit: impl FnOnce(&mut format_job_tree) -> R,
    ) -> Option<R> {
        unsafe { (*self.0.value.as_ptr()).jobs.as_deref_mut().map(visit) }
    }

    /// Creates a format job entry only when the client has no entry for its key.
    ///
    /// # Safety
    /// Exclude other cache access, including from `create`, during this call.
    pub(crate) unsafe fn ensure_format_job(
        &mut self,
        key: (u_int, std::ffi::CString),
        create: impl FnOnce() -> Box<format_job>,
    ) {
        unsafe {
            (*self.0.value.as_ptr())
                .jobs
                .get_or_insert_with(|| Box::new(format_job_tree::new()))
                .entry(key)
                .or_insert_with(create);
        }
    }

    /// Schedules a redraw of the client's status line.
    ///
    /// # Safety
    /// The flags must not otherwise be accessed during this call.
    pub(crate) unsafe fn request_status_redraw(&mut self) {
        unsafe {
            *self.flags_mut() |= crate::status::CLIENT_REDRAWSTATUS as uint64_t;
        }
    }

    /// Reads the client's discarded.
    pub(crate) fn discarded(&self) -> size_t {
        self.0.value.borrow().discarded
    }

    /// Reads the client's written.
    pub(crate) fn written(&self) -> size_t {
        self.0.value.borrow().written
    }

    /// Reads the client's pid.
    pub(crate) fn pid(&self) -> pid_t {
        self.0.value.borrow().pid
    }

    /// Reads the client's theme.
    pub(crate) fn theme(&self) -> client_theme {
        self.0.value.borrow().theme
    }

    /// Reads the client's creation time.
    pub(crate) fn creation_time(&self) -> timeval {
        self.0.value.borrow().creation_time
    }

    /// Copies the client's terminal name.
    pub(crate) fn terminal_name(&self) -> Option<std::ffi::CString> {
        self.0.value.borrow().term_name.clone()
    }

    /// Copies the client's terminal type.
    pub(crate) fn terminal_type(&self) -> Option<std::ffi::CString> {
        self.0.value.borrow().term_type.clone()
    }

    /// Copies terminal dimensions in cells using a checked client borrow.
    ///
    /// No callbacks run and no client or terminal borrow escapes the snapshot.
    pub(crate) fn terminal_size(&self) -> crate::pane_resize::PaneSize {
        let client = self.0.value.borrow();
        crate::pane_resize::PaneSize {
            width: client.tty.sx,
            height: client.tty.sy,
        }
    }

    /// Copies the cached window visibility flag, offsets and visible size.
    ///
    /// This checked snapshot does not recalculate offsets or run callbacks. No
    /// client or terminal borrow escapes the call.
    pub(crate) fn cached_window_offset(&self) -> (core::ffi::c_int, u_int, u_int, u_int, u_int) {
        let client = self.0.value.borrow();
        let tty = &client.tty;
        (tty.oflag, tty.oox, tty.ooy, tty.osx, tty.osy)
    }

    /// Resolves and caches the connected peer's user name.
    ///
    /// # Safety
    /// Exclude conflicting access to the peer and user cache during this call.
    pub(crate) unsafe fn user_name(&mut self) -> Option<std::ffi::CString> {
        use crate::UserAccount;

        unsafe {
            if let Some(user) = (*self.0.value.as_ptr()).user.clone() {
                return Some(user);
            }
            let uid = self.peer_handle().uid();
            if uid == uid_t::MAX {
                return None;
            }
            let account = crate::UserAccountRecord::lookup_uid(uid)?;
            let user = account.account_name().map(std::ffi::CStr::to_owned);
            (*self.0.value.as_ptr()).user = user.clone();
            user
        }
    }

    /// Borrows the client's exit message state exclusively.
    ///
    /// # Safety
    /// The caller must exclude other access to this field for the borrow's lifetime.
    pub(crate) unsafe fn exit_message_mut(&mut self) -> &mut Option<std::ffi::CString> {
        unsafe { &mut (*self.0.value.as_ptr()).exit_message }
    }

    /// Borrows the client's status state.
    ///
    /// # Safety
    /// The caller must exclude mutation of this field for the borrow's lifetime.
    pub(crate) unsafe fn status_ref(&self) -> &status_line {
        unsafe { &(*self.0.value.as_ptr()).status }
    }

    /// Retains the client's command queue.
    pub(crate) fn command_queue(&self) -> crate::cmd::CmdqListRef {
        self.0
            .value
            .borrow()
            .queue
            .as_ref()
            .expect("the client has a command queue")
            .clone()
    }

    /// Borrows the client's exit status exclusively.
    ///
    /// # Safety
    /// The caller must exclude other access to this field for the borrow's lifetime.
    pub(crate) unsafe fn retval_mut(&mut self) -> &mut core::ffi::c_int {
        unsafe { &mut (*self.0.value.as_ptr()).retval }
    }

    /// Reads the client's source file depth.
    pub(crate) fn source_file_depth(&self) -> u_int {
        self.0.value.borrow().source_file_depth
    }

    /// Reads the client's prompt.
    pub(crate) fn prompt(&self) -> Prompt {
        self.0.value.borrow().prompt
    }

    /// Reads the client's activity time.
    pub(crate) fn activity_time(&self) -> timeval {
        self.0.value.borrow().activity_time
    }

    /// Borrows the client's terminal name.
    ///
    /// # Safety
    /// The field must not be mutated for the returned borrow's lifetime.
    pub(crate) unsafe fn ttyname_ref(&self) -> &Option<std::ffi::CString> {
        unsafe { &(*self.0.value.as_ptr()).ttyname }
    }

    /// The overlay the client is showing, which is `Overlay::None` when it is
    /// showing none.
    pub(crate) fn overlay(&self) -> Overlay {
        self.0.value.borrow().overlay
    }

    /// Which overlay answers for the region the current drawing may cover.
    pub(crate) fn overlay_check(&self) -> OverlayCheck {
        self.0.value.borrow().overlay_check
    }

    /// Returns the drawing data for the current overlay view.
    ///
    /// # Safety
    /// The accessed fields must not otherwise be accessed during this call.
    pub(crate) unsafe fn current_overlay_data(&mut self) -> OverlayData {
        unsafe {
            (*self.0.value.as_ptr())
                .overlay_data
                .view_data((*self.0.value.as_ptr()).overlay_data_view)
        }
    }

    /// Points the region at `view` for the drawing that follows, which is how
    /// a popup hands it to the menu it carries and takes it back afterwards.
    ///
    /// # Safety
    /// The accessed fields must not otherwise be accessed during this call.
    pub(crate) unsafe fn set_overlay_view(&mut self, view: OverlayView) {
        unsafe {
            match view {
                OverlayView::Menu => {
                    (*self.0.value.as_ptr()).overlay_check = OverlayCheck::Menu;
                    (*self.0.value.as_ptr()).overlay_data_view = Some(OverlayView::Menu);
                }
                OverlayView::Nothing => {
                    (*self.0.value.as_ptr()).overlay_check = OverlayCheck::None;
                    (*self.0.value.as_ptr()).overlay_data_view = Some(OverlayView::Nothing);
                }
                OverlayView::Popup => {
                    (*self.0.value.as_ptr()).overlay_check = OverlayCheck::Popup;
                    (*self.0.value.as_ptr()).overlay_data_view = None;
                }
            }
        }
    }

    /// Gives the drawing data back to the overlay itself, which is what a
    /// popup does once the menu it was carrying is gone.
    ///
    /// # Safety
    /// The accessed fields must not otherwise be accessed during this call.
    pub(crate) unsafe fn clear_overlay_view(&mut self) {
        unsafe {
            (*self.0.value.as_ptr()).overlay_data_view = None;
        }
    }

    /// Borrows the environment the client was started with.
    ///
    /// # Safety
    /// The accessed fields must not be exclusively borrowed during this call or
    /// for the returned borrow's lifetime.
    pub(crate) unsafe fn environ_ref(&self) -> &RustEnvironment {
        unsafe {
            (*self.0.value.as_ptr())
                .environ
                .as_deref()
                .expect("a client is created with its environment")
        }
    }

    /// Borrows the client environment exclusively.
    ///
    /// # Safety
    /// The accessed fields must not otherwise be accessed during this call or for
    /// the returned borrow's lifetime.
    pub(crate) unsafe fn environ_mut(&mut self) -> &mut RustEnvironment {
        unsafe {
            (*self.0.value.as_ptr())
                .environ
                .as_deref_mut()
                .expect("a client is created with its environment")
        }
    }

    /// Retains the peer handle carrying the connected client's messages.
    pub(crate) fn peer_handle(&self) -> PeerRef {
        self.0
            .value
            .borrow()
            .peer
            .as_ref()
            .expect("a connected client has a peer")
            .clone()
    }

    /// The key table the client's next key is looked up in, if one is assigned.
    pub(crate) fn keytable(&self) -> Option<KeyTableRef> {
        self.0.value.borrow().keytable_ref.clone()
    }

    /// Retains the observed attachment while its session is still alive.
    pub(crate) fn attached_session(&self) -> Option<SessionRef> {
        self.0
            .value
            .borrow()
            .attached_session
            .as_ref()
            .and_then(SessionWeak::upgrade)
    }

    /// Records the attachment without running session-change callbacks or redraws.
    ///
    /// # Safety
    /// The accessed fields must not otherwise be accessed during this call.
    pub(crate) unsafe fn set_attached_session(&mut self, session: Option<&SessionRef>) {
        unsafe {
            (*self.0.value.as_ptr()).attached_session = session.map(SessionRef::downgrade);
        }
    }

    pub(crate) fn new(value: client) -> Self {
        let reference = Self(Rc::new(ClientStorage {
            id: NEXT_CLIENT_ID.with(|next| {
                let id = next.get();
                next.set(id.checked_add(1).expect("client IDs exhausted"));
                id
            }),
            value: RefCell::new(value),
        }));
        reference.0.value.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    /// Returns a compatibility view of the client payload.
    ///
    /// # Safety
    /// The caller must prevent mutation for the reference's lifetime, including
    /// mutation through other owners or reentrant callbacks.
    pub(crate) unsafe fn as_client(&self) -> &client {
        unsafe { &*self.0.value.as_ptr() }
    }

    /// Returns an exclusive compatibility view of the client payload.
    ///
    /// # Safety
    /// The caller must exclude all other access for this borrow's lifetime,
    /// including access through other owners or reentrant callbacks.
    pub(crate) unsafe fn as_client_mut(&mut self) -> &mut client {
        unsafe { &mut *self.0.value.as_ptr() }
    }

    /// Borrows the terminal without borrowing the other client fields.
    ///
    /// # Safety
    /// The terminal must not be mutated for the returned borrow's lifetime.
    pub(crate) unsafe fn as_tty(&self) -> &tty {
        unsafe { &(*self.0.value.as_ptr()).tty }
    }

    /// Borrows the terminal exclusively without borrowing other client fields.
    ///
    /// # Safety
    /// The caller must exclude other terminal access for this borrow,
    /// including access through other owners or reentrant callbacks.
    pub(crate) unsafe fn as_tty_mut(&mut self) -> &mut tty {
        unsafe { &mut (*self.0.value.as_ptr()).tty }
    }

    /// Borrows the display name without borrowing the client's terminal.
    ///
    /// # Safety
    /// The name must not be mutated for the returned borrow's lifetime.
    pub(crate) unsafe fn name(&self) -> Option<&std::ffi::CStr> {
        unsafe { (*self.0.value.as_ptr()).name.as_deref() }
    }

    /// Reads the terminal descriptor without borrowing the client's terminal.
    ///
    /// # Safety
    /// The descriptor must not be exclusively borrowed during this read.
    pub(crate) unsafe fn fd(&self) -> core::ffi::c_int {
        unsafe { (*self.0.value.as_ptr()).fd }
    }

    /// Borrows the client flags independently of the terminal.
    ///
    /// # Safety
    /// The caller must exclude other access to the flags for this borrow.
    pub(crate) unsafe fn flags_mut(&mut self) -> &mut uint64_t {
        unsafe { &mut (*self.0.value.as_ptr()).flags }
    }

    /// Reads the client flags without borrowing the terminal.
    ///
    /// # Safety
    /// The flags must not be exclusively borrowed during this read.
    pub(crate) unsafe fn flags(&self) -> uint64_t {
        unsafe { (*self.0.value.as_ptr()).flags }
    }

    /// Records queued terminal output without borrowing the terminal itself.
    ///
    /// # Safety
    /// The written-byte count must not otherwise be accessed during this call.
    pub(crate) unsafe fn record_written(&mut self, count: size_t) {
        unsafe {
            let written = &mut (*self.0.value.as_ptr()).written;
            *written = written.wrapping_add(count);
        }
    }

    /// Borrows the discarded-output count independently of the terminal.
    ///
    /// # Safety
    /// The caller must exclude other access to the count for this borrow.
    pub(crate) unsafe fn discarded_mut(&mut self) -> &mut size_t {
        unsafe { &mut (*self.0.value.as_ptr()).discarded }
    }

    /// Borrows the pending-redraw count independently of the terminal.
    ///
    /// # Safety
    /// The caller must exclude other access to the count for this borrow.
    pub(crate) unsafe fn redraw_mut(&mut self) -> &mut size_t {
        unsafe { &mut (*self.0.value.as_ptr()).redraw }
    }

    /// Borrows pending input requests independently of the terminal.
    ///
    /// # Safety
    /// The caller must exclude other access to the list for this borrow.
    pub(crate) unsafe fn input_requests_mut(&mut self) -> &mut input_requests {
        unsafe { &mut (*self.0.value.as_ptr()).input_requests }
    }

    /// Reads the terminal features without borrowing the terminal itself.
    ///
    /// # Safety
    /// The features must not be exclusively borrowed during this read.
    pub(crate) unsafe fn terminal_features(&self) -> core::ffi::c_int {
        unsafe { (*self.0.value.as_ptr()).term_features }
    }

    /// Borrows the terminal features independently of the terminal.
    ///
    /// # Safety
    /// The caller must exclude other access to the features for this borrow.
    pub(crate) unsafe fn terminal_features_mut(&mut self) -> &mut core::ffi::c_int {
        unsafe { &mut (*self.0.value.as_ptr()).term_features }
    }

    /// Borrows the reported terminal type independently of the terminal.
    ///
    /// # Safety
    /// The caller must exclude other access to the type for this borrow.
    pub(crate) unsafe fn terminal_type_mut(&mut self) -> &mut Option<std::ffi::CString> {
        unsafe { &mut (*self.0.value.as_ptr()).term_type }
    }

    /// Borrows the terminal definition without borrowing the terminal itself.
    ///
    /// # Safety
    /// The name and capabilities must not be mutated, and the features must
    /// not otherwise be accessed, for the lifetime of the returned borrows.
    pub(crate) unsafe fn terminal_definition(
        &mut self,
    ) -> (
        Option<&std::ffi::CStr>,
        &[std::ffi::CString],
        &mut core::ffi::c_int,
    ) {
        unsafe {
            (
                (*self.0.value.as_ptr()).term_name.as_deref(),
                &(*self.0.value.as_ptr()).term_caps,
                &mut (*self.0.value.as_ptr()).term_features,
            )
        }
    }

    /// Checks for an overlay without borrowing the client's terminal.
    ///
    /// # Safety
    /// The overlay state must not be exclusively borrowed during this read.
    pub(crate) unsafe fn has_overlay(&self) -> bool {
        unsafe { (*self.0.value.as_ptr()).overlay.is_some() }
    }

    /// Checks for an overlay region callback without borrowing the terminal.
    ///
    /// # Safety
    /// The overlay check must not be exclusively borrowed during this read.
    pub(crate) unsafe fn has_overlay_check(&self) -> bool {
        unsafe { (*self.0.value.as_ptr()).overlay_check.is_some() }
    }

    /// Queries the overlay's visible ranges without borrowing the terminal.
    ///
    /// # Safety
    /// The caller must exclude other access to the overlay data while its
    /// region callback runs. Other client fields may remain borrowed.
    pub(crate) unsafe fn overlay_ranges(
        &mut self,
        px: u_int,
        py: u_int,
        nx: u_int,
    ) -> Option<VisibleRangesRef> {
        unsafe {
            let check = (*self.0.value.as_ptr()).overlay_check;
            if check.is_none() {
                return None;
            }
            let view = (*self.0.value.as_ptr()).overlay_data_view;
            let data = (*self.0.value.as_ptr()).overlay_data.view_data(view);
            Some(check.call(data, px, py, nx))
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut client {
        self.0.value.as_ptr()
    }

    pub(crate) fn id(&self) -> u64 {
        self.0.id
    }

    pub(crate) fn downgrade(&self) -> ClientWeak {
        ClientWeak(Rc::downgrade(&self.0))
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl ClientWeak {
    pub(crate) fn upgrade(&self) -> Option<ClientRef> {
        self.0.upgrade().map(ClientRef)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

/// A shared key-binding table with checked access to its state.
#[derive(Clone)]
pub(crate) struct KeyTableRef(Rc<RefCell<key_table>>);

impl KeyTableRef {
    pub(crate) fn new(value: key_table) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, key_table> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, key_table> {
        self.0.borrow_mut()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq for KeyTableRef {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for KeyTableRef {}

impl fmt::Debug for KeyTableRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("KeyTableRef")
            .field(&Rc::as_ptr(&self.0))
            .finish()
    }
}

/// Shared ownership of a client file operation with checked state access.
#[derive(Clone)]
pub(crate) struct ClientFileRef(Rc<RefCell<client_file>>);

impl ClientFileRef {
    pub(crate) fn new(value: client_file) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, client_file> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, client_file> {
        self.0.borrow_mut()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub struct ModeTreeDataRef(Rc<ModeTreeStorage>);

struct ModeTreeStorage {
    state: RefCell<mode_tree_data>,
    screen: ScreenRef,
}

#[derive(Clone)]
pub struct ModeTreeDataWeak(Weak<ModeTreeStorage>);

impl ModeTreeDataRef {
    pub(crate) fn new(value: mode_tree_data, screen: ScreenRef) -> Self {
        Self(Rc::new(ModeTreeStorage {
            state: RefCell::new(value),
            screen,
        }))
    }

    /// The tree's screen, independently accessible from its item state.
    pub(crate) fn screen_handle(&self) -> &ScreenRef {
        &self.0.screen
    }

    pub(crate) fn set_default_cursor(&self, options: &RustOptionsRef) {
        self.0.screen.borrow_mut().set_default_cursor(options)
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, mode_tree_data> {
        self.0.state.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, mode_tree_data> {
        self.0.state.borrow_mut()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn downgrade(&self) -> ModeTreeDataWeak {
        ModeTreeDataWeak(Rc::downgrade(&self.0))
    }
}

impl ModeTreeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<ModeTreeDataRef> {
        self.0.upgrade().map(ModeTreeDataRef)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub struct WindowBufferModeDataRef(Rc<RefCell<window_buffer_modedata>>);

#[derive(Clone)]
pub struct WindowBufferModeDataWeak(Weak<RefCell<window_buffer_modedata>>);

impl WindowBufferModeDataRef {
    pub(crate) fn new(value: window_buffer_modedata) -> Self {
        let reference = Self(Rc::new(RefCell::new(value)));
        reference.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, window_buffer_modedata> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, window_buffer_modedata> {
        self.0.borrow_mut()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn downgrade(&self) -> WindowBufferModeDataWeak {
        WindowBufferModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowBufferModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowBufferModeDataRef> {
        self.0.upgrade().map(WindowBufferModeDataRef)
    }
}

#[derive(Clone)]
pub struct WindowClientModeDataRef(Rc<RefCell<window_client_modedata>>);

#[derive(Clone)]
pub struct WindowClientModeDataWeak(Weak<RefCell<window_client_modedata>>);

impl WindowClientModeDataRef {
    pub(crate) fn new(value: window_client_modedata) -> Self {
        let reference = Self(Rc::new(RefCell::new(value)));
        reference.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, window_client_modedata> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, window_client_modedata> {
        self.0.borrow_mut()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn downgrade(&self) -> WindowClientModeDataWeak {
        WindowClientModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowClientModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowClientModeDataRef> {
        self.0.upgrade().map(WindowClientModeDataRef)
    }
}

#[derive(Clone)]
pub struct WindowTreeModeDataRef(Rc<RefCell<window_tree_modedata>>);

#[derive(Clone)]
pub struct WindowTreeModeDataWeak(Weak<RefCell<window_tree_modedata>>);

impl WindowTreeModeDataRef {
    pub(crate) fn new(value: window_tree_modedata) -> Self {
        let reference = Self(Rc::new(RefCell::new(value)));
        reference.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, window_tree_modedata> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, window_tree_modedata> {
        self.0.borrow_mut()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn downgrade(&self) -> WindowTreeModeDataWeak {
        WindowTreeModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowTreeModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowTreeModeDataRef> {
        self.0.upgrade().map(WindowTreeModeDataRef)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub struct WindowCustomizeModeDataRef(Rc<RefCell<window_customize_modedata>>);

#[derive(Clone)]
pub struct WindowCustomizeModeDataWeak(Weak<RefCell<window_customize_modedata>>);

impl WindowCustomizeModeDataRef {
    pub(crate) fn new(value: window_customize_modedata) -> Self {
        let reference = Self(Rc::new(RefCell::new(value)));
        reference.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, window_customize_modedata> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, window_customize_modedata> {
        self.0.borrow_mut()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn downgrade(&self) -> WindowCustomizeModeDataWeak {
        WindowCustomizeModeDataWeak(Rc::downgrade(&self.0))
    }
}

impl WindowCustomizeModeDataWeak {
    pub(crate) fn upgrade(&self) -> Option<WindowCustomizeModeDataRef> {
        self.0.upgrade().map(WindowCustomizeModeDataRef)
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

pub type cmd_parse_status = core::ffi::c_uint;
#[repr(C)]
pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub(crate) cmdlist: Option<CmdListRef>,
    pub error: Option<std::ffi::CString>,
}
impl Default for cmd_parse_result {
    /// The `CMD_PARSE_SUCCESS` status with neither a command list nor an error.
    fn default() -> cmd_parse_result {
        cmd_parse_result {
            status: 0,
            cmdlist: None,
            error: None,
        }
    }
}
pub use crate::style::colour_palette;
pub type control_sub_type = core::ffi::c_uint;
pub type size_t = usize;
pub type format_entry_cb = Option<unsafe fn(&format_tree) -> Option<std::ffi::CString>>;
pub type gid_t = __gid_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct stat_t {
    pub st_dev: __dev_t,
    pub st_ino: __ino_t,
    pub st_nlink: __nlink_t,
    pub st_mode: __mode_t,
    pub st_uid: __uid_t,
    pub st_gid: __gid_t,
    pub __pad0: core::ffi::c_int,
    pub st_rdev: __dev_t,
    pub st_size: __off_t,
    pub st_blksize: __blksize_t,
    pub st_blocks: __blkcnt_t,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    pub __glibc_reserved: [__syscall_slong_t; 3],
}
pub type u_char = __u_char;
pub type u_short = __u_short;
pub use crate::text::utf8_data;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct grid_cell {
    pub data: utf8_data,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: core::ffi::c_int,
    pub bg: core::ffi::c_int,
    pub us: core::ffi::c_int,
    pub link: u_int,
}
pub use crate::text::utf8_char;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_extd_entry {
    pub data: utf8_char,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: core::ffi::c_int,
    pub bg: core::ffi::c_int,
    pub us: core::ffi::c_int,
    pub link: u_int,
}
pub use crate::text::hanguljamo_state;
pub type uint32_t = __uint32_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct imsg_hdr {
    pub type_0: uint32_t,
    pub len: uint32_t,
    pub peerid: uint32_t,
    pub pid: uint32_t,
}
pub type pid_t = __pid_t;
#[repr(C)]
pub struct imsgbuf {
    pub w: Option<Box<msgbuf>>,
    pub pid: pid_t,
    pub maxsize: uint32_t,
    pub fd: core::ffi::c_int,
    pub flags: core::ffi::c_int,
}
pub type in_addr_t = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}
pub type uint16_t = __uint16_t;
pub type in_port_t = uint16_t;
/// What a client read back off the terminal, for the parser that asked.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum InputRequestData {
    #[default]
    None,
    Palette {
        idx: core::ffi::c_int,
        c: core::ffi::c_int,
    },
    Clipboard {
        clip: core::ffi::c_char,
        data: Vec<u8>,
    },
}
pub type input_request_type = core::ffi::c_uint;
/// The requests one parser is waiting on a reply to, oldest first. A request
/// belongs to the parser that made it until its request handle is freed.
pub type input_request_list = Vec<Box<input_request>>;
/// The requests one client is answering, oldest first. The parser that made
/// each one owns it.
pub type input_requests = Vec<input_request_handle>;
pub type job_complete_cb = Option<Box<dyn FnOnce(JobEvent)>>;
pub type job_update_cb = Option<std::rc::Rc<dyn Fn(JobEvent)>>;
pub use crate::text::{key_code, key_code_type};
pub type tty_code_type = core::ffi::c_uint;
/// A mouse event. The default is an invalid one, which is what a client
/// starts out holding.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct mouse_event {
    pub valid: core::ffi::c_int,
    pub ignore: core::ffi::c_int,
    pub key: key_code,
    pub statusat: core::ffi::c_int,
    pub statuslines: u_int,
    pub x: u_int,
    pub y: u_int,
    pub b: u_int,
    pub lx: u_int,
    pub ly: u_int,
    pub lb: u_int,
    pub ox: u_int,
    pub oy: u_int,
    pub s: core::ffi::c_int,
    pub w: core::ffi::c_int,
    pub wp: core::ffi::c_int,
    pub sgr_type: u_int,
    pub sgr_b: u_int,
}
#[derive(Clone)]
#[repr(C)]
#[derive(Default)]
pub struct key_event {
    pub key: key_code,
    pub m: mouse_event,
    pub buf: Vec<u8>,
}
pub type layout_type = core::ffi::c_uint;
/// One item of a menu template. The default is the separator item: no name,
/// no key and no command.
/// One item a caller hands to `menu_add_item`. Its strings are borrowed:
/// from the `static` template a mode keeps, or from whatever the command
/// that is building the menu owns.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct menu_item<'a> {
    /// What the item reads as, which is expanded as a format before it is
    /// drawn, or nothing for the separator line.
    pub name: Option<&'a core::ffi::CStr>,
    pub key: key_code,
    /// The command the item runs, or nothing for one whose caller runs its
    /// own.
    pub command: Option<&'a core::ffi::CStr>,
}

/// One line of a menu as the menu itself holds it, which is not the same
/// thing as the [`menu_item`] a caller hands to `menu_add_item`: a template's
/// strings are borrowed — often from a `static` array — while a stored
/// entry's name is the expansion the menu made and owns.
#[repr(C)]
pub struct menu_entry {
    pub name: Option<std::ffi::CString>,
    pub key: key_code,
    pub command: Option<std::ffi::CString>,
}
impl menu_entry {
    /// Whether the entry is a named choice rather than a separator or disabled row.
    pub(crate) fn is_selectable(&self) -> bool {
        self.name()
            .is_some_and(|name| !name.to_bytes().starts_with(b"-"))
    }

    /// The text the menu draws for the entry, or nothing for a separator.
    pub(crate) fn name(&self) -> Option<&core::ffi::CStr> {
        self.name.as_deref()
    }

    /// The command the entry runs when it is chosen, or nothing for one that
    /// runs none.
    pub(crate) fn command(&self) -> Option<&core::ffi::CStr> {
        self.command.as_deref()
    }
}
impl Default for menu_entry {
    /// The separator line: no name, no key and no command.
    fn default() -> menu_entry {
        menu_entry {
            name: None,
            key: 0,
            command: None,
        }
    }
}
#[repr(C)]
pub struct menu {
    pub title: Option<std::ffi::CString>,
    pub items: Vec<menu_entry>,
    pub width: u_int,
}

pub type menu_choice_cb = Option<Box<dyn FnOnce(u_int, key_code)>>;
pub type sort_order = core::ffi::c_uint;
pub type sort_criteria_t = crate::sort::RustSortCriteria;
pub type uint64_t = __uint64_t;
pub type mode_tree_build_cb =
    Option<Rc<dyn Fn(&sort_criteria_t, &mut uint64_t, Option<&core::ffi::CStr>)>>;
pub type mode_tree_height_cb = Option<Rc<dyn Fn() -> u_int>>;
/// The help a mode shows: its own lines, the least width they need, and the
/// word `%1` stands for in every line.
pub type mode_tree_help = Option<(
    &'static [&'static core::ffi::CStr],
    u_int,
    &'static core::ffi::CStr,
)>;
pub type mode_tree_key_cb = Option<Rc<dyn Fn(ModeTreeItemData, u_int) -> key_code>>;
pub type mode_tree_search_cb =
    Option<Rc<dyn Fn(ModeTreeItemData, &core::ffi::CStr, core::ffi::c_int) -> core::ffi::c_int>>;
pub type mode_tree_sort_cb = Option<Rc<dyn Fn(&mut sort_criteria_t)>>;
pub type mode_tree_swap_cb =
    Option<Rc<dyn Fn(ModeTreeItemData, ModeTreeItemData, &sort_criteria_t) -> core::ffi::c_int>>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_command {
    pub argc: core::ffi::c_int,
}
pub type msgtype = core::ffi::c_uint;
pub type nl_item = core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_name_map {
    pub from: &'static core::ffi::CStr,
    pub to: &'static core::ffi::CStr,
}
pub type options_table_type = core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_table_entry_t {
    pub name: &'static core::ffi::CStr,
    pub alternative_name: Option<&'static core::ffi::CStr>,
    pub type_0: options_table_type,
    pub scope: core::ffi::c_int,
    pub flags: core::ffi::c_int,
    pub minimum: u_int,
    pub maximum: u_int,
    pub choices: Option<&'static [&'static core::ffi::CStr]>,
    pub default_str: Option<&'static core::ffi::CStr>,
    pub default_num: core::ffi::c_longlong,
    pub default_arr: Option<&'static [&'static core::ffi::CStr]>,
    pub separator: Option<&'static core::ffi::CStr>,
    pub pattern: Option<&'static core::ffi::CStr>,
    pub text: Option<&'static core::ffi::CStr>,
    pub unit: Option<&'static core::ffi::CStr>,
}
pub type style_align = core::ffi::c_uint;
pub type style_default_type = core::ffi::c_uint;
pub type style_list = core::ffi::c_uint;
pub type style_range_type = core::ffi::c_uint;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct style {
    pub gc: grid_cell,
    pub ignore: core::ffi::c_int,
    pub fill: core::ffi::c_int,
    pub align: style_align,
    pub list: style_list,
    pub range_type: style_range_type,
    pub range_argument: u_int,
    pub range_string: [core::ffi::c_char; 16],
    pub width: core::ffi::c_int,
    pub width_percentage: core::ffi::c_int,
    pub pad: core::ffi::c_int,
    pub default_type: style_default_type,
}
pub use crate::options::options_value;
pub type pane_lines = core::ffi::c_uint;
pub type popup_finish_edit_cb = Box<dyn FnOnce(Vec<u8>)>;
pub type progress_bar_state = core::ffi::c_uint;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct progress_bar {
    pub state: progress_bar_state,
    pub progress: core::ffi::c_int,
}
pub type prompt_type = core::ffi::c_uint;
pub type regoff_t = core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}
pub type sa_family_t = core::ffi::c_ushort;
pub type screen_cursor_style = core::ffi::c_uint;
pub type sigset_t = __sigset_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [core::ffi::c_char; 14],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [core::ffi::c_uchar; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [u8; 108],
}
impl Default for sockaddr_un {
    /// An address in no family, with an empty path.
    fn default() -> sockaddr_un {
        sockaddr_un {
            sun_family: 0,
            sun_path: [0; 108],
        }
    }
}
pub type socklen_t = __socklen_t;
pub type speed_t = core::ffi::c_uint;
pub type ssize_t = isize;
pub type tcflag_t = core::ffi::c_uint;
pub type time_t = __time_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
impl timeval {
    /// A span of whole seconds.
    pub const fn from_secs(tv_sec: __time_t) -> timeval {
        timeval { tv_sec, tv_usec: 0 }
    }
    /// A span of microseconds, which the callers here keep under a second.
    pub const fn from_usecs(tv_usec: __suseconds_t) -> timeval {
        timeval { tv_sec: 0, tv_usec }
    }
}
#[derive(Clone, Default)]
pub struct tm {
    pub tm_sec: core::ffi::c_int,
    pub tm_min: core::ffi::c_int,
    pub tm_hour: core::ffi::c_int,
    pub tm_mday: core::ffi::c_int,
    pub tm_mon: core::ffi::c_int,
    pub tm_year: core::ffi::c_int,
    pub tm_wday: core::ffi::c_int,
    pub tm_yday: core::ffi::c_int,
    pub tm_isdst: core::ffi::c_int,
    pub tm_gmtoff: core::ffi::c_long,
    pub tm_zone: Option<std::rc::Rc<core::ffi::CStr>>,
}
pub type tty_code_code = core::ffi::c_uint;
pub type uid_t = __uid_t;
pub type uint8_t = __uint8_t;
pub use crate::text::utf8_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visible_range {
    pub px: u_int,
    pub nx: u_int,
}
#[derive(Default)]
#[repr(C)]
pub struct visible_ranges {
    pub ranges: Vec<visible_range>,
    pub used: u_int,
}
/// Shared scratch storage for overlay clipping. Drawing may check the overlay
/// again, so each span is read with a borrow that ends before output begins.
#[derive(Clone, Default)]
pub struct VisibleRangesRef(Rc<RefCell<visible_ranges>>);

impl VisibleRangesRef {
    pub fn borrow(&self) -> std::cell::Ref<'_, visible_ranges> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, visible_ranges> {
        self.0.borrow_mut()
    }

    pub fn range_at(&self, index: u_int) -> Option<visible_range> {
        let ranges = self.borrow();
        (index < ranges.used).then(|| ranges.ranges[index as usize])
    }
}
pub type wchar_t = libc::wchar_t;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct winsize {
    pub ws_row: core::ffi::c_ushort,
    pub ws_col: core::ffi::c_ushort,
    pub ws_xpixel: core::ffi::c_ushort,
    pub ws_ypixel: core::ffi::c_ushort,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_open {
    pub stream: core::ffi::c_int,
    pub fd: core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_data {
    pub stream: core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct msg_read_done {
    pub stream: core::ffi::c_int,
    pub error: core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_cancel {
    pub stream: core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct msg_write_open {
    pub stream: core::ffi::c_int,
    pub fd: core::ffi::c_int,
    pub flags: core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_data {
    pub stream: core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct msg_write_ready {
    pub stream: core::ffi::c_int,
    pub error: core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_close {
    pub stream: core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_code {
    pub code: tty_code_code,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_raw {
    /// The bytes the terminal sends for the key.
    pub string: &'static core::ffi::CStr,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_xterm {
    /// The bytes the terminal sends for the key, with `_` standing where the
    /// modifier digit goes.
    pub template: &'static core::ffi::CStr,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_term_code_entry {
    pub type_0: tty_code_type,
    /// The terminfo name of the capability.
    pub name: &'static core::ffi::CStr,
}
impl tty_term_code_entry {
    /// Builds a static terminal capability table entry.
    pub const fn new(type_0: tty_code_type, name: &'static core::ffi::CStr) -> Self {
        Self { type_0, name }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_cell {
    pub cell: grid_cell,
    pub set: core::ffi::c_int,
    pub g0set: core::ffi::c_int,
    pub g1set: core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_table_entry {
    pub ch: core::ffi::c_int,
    /// The intermediate bytes the sequence carries before its final byte.
    pub interm: &'static core::ffi::CStr,
    pub type_0: core::ffi::c_int,
}

#[derive(Clone)]
#[repr(C)]
pub struct cmd_command_prompt_prompt {
    pub input: Option<std::ffi::CString>,
    pub prompt: Option<std::ffi::CString>,
}
#[derive(Clone, Default)]
#[repr(C)]
pub struct window_buffer_itemdata {
    pub name: Option<std::ffi::CString>,
    pub order: u_int,
    pub size: size_t,
}
impl window_buffer_itemdata {
    /// The name of the paste buffer the row stands for.
    #[cfg(test)]
    pub(crate) fn name(&self) -> Option<&core::ffi::CStr> {
        self.name.as_deref()
    }
}
#[repr(C)]
pub struct format_modifier {
    pub modifier: [u8; 3],
    pub size: u_int,
    pub argv: Vec<std::ffi::CString>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid_cell_entry_data {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union grid_cell_entry_union {
    pub offset: u_int,
    pub data: grid_cell_entry_data,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_cell_entry {
    pub c2rust_unnamed: grid_cell_entry_union,
    pub flags: u_char,
}
const _: () = assert!(size_of::<grid_cell_entry>() == 5);
#[repr(C)]
pub struct grid {
    pub flags: core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub hscrolled: u_int,
    pub hsize: u_int,
    pub hlimit: u_int,
    pub linedata: Vec<grid_line>,
}
pub use crate::grid::grid_line;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct style_range {
    pub type_0: style_range_type,
    pub argument: u_int,
    pub string: [core::ffi::c_char; 16],
    pub start: u_int,
    pub end: u_int,
}
pub type style_ranges = Vec<style_range>;
#[derive(Default)]
pub struct style_line_entry {
    pub expanded: Option<std::ffi::CString>,
    pub ranges: style_ranges,
}
pub type client_exit_type = core::ffi::c_uint;
pub type client_prompt_mode = core::ffi::c_uint;
#[repr(C)]
pub struct winlink {
    pub idx: core::ffi::c_int,
    /// The session that holds the link, observed rather than held: the
    /// session owns the link, so holding it back would be a cycle.
    pub(crate) session_ref: Option<SessionWeak>,
    pub(crate) window_ref: Option<WindowRef>,
    pub flags: core::ffi::c_int,
}

impl crate::WinlinkIdentity for winlink {
    fn winlink_index(&self) -> core::ffi::c_int {
        self.idx
    }

    fn set_winlink_index(&mut self, index: core::ffi::c_int) {
        self.idx = index;
    }
}

impl crate::WinlinkFlagsState for winlink {
    fn winlink_flags(&self) -> core::ffi::c_int {
        self.flags
    }

    fn set_winlink_flags(&mut self, flags: core::ffi::c_int) {
        self.flags = flags;
    }
}

impl winlink {
    /// The session that holds this link, or null once it has gone.
    pub(crate) fn session(&self) -> Option<SessionRef> {
        self.session_ref.as_ref().and_then(SessionWeak::upgrade)
    }

    /// The handle on the window this link points at, borrowed for as long as
    /// the link is, or nothing while the link holds none.
    pub(crate) fn window_handle(&self) -> Option<&WindowRef> {
        self.window_ref.as_ref()
    }
}
#[derive(Default)]
#[repr(C)]
pub struct window {
    pub(crate) owner: Option<WindowWeak>,
    pub(crate) id: u32,
    pub(crate) latest: Option<ClientWeak>,
    pub(crate) name_state: crate::window_name::RustWindowNameState,
    pub name_event: TimerHandle,
    pub(crate) timestamps: crate::window_timestamps::RustWindowTimestampState,
    pub alerts_timer: TimerHandle,
    pub offset_timer: TimerHandle,
    /// The pane the window is showing as its active one.
    pub active_pane: Option<RustWindowPaneWeak>,
    pub last_panes: window_pane_stack_t,
    pub z_index: window_pane_stack_t,
    pub(crate) panes: window_panes_t,
    pub(crate) layout_selection: crate::window_layout_selection::RustWindowLayoutSelectionState,
    pub layout_root: Option<Box<layout_cell>>,
    pub saved_layout_root: Option<Box<layout_cell>>,
    pub(crate) saved_layout_state: crate::window_saved_layout::RustWindowSavedLayoutState,
    pub(crate) dimensions: crate::window_dimensions::RustWindowDimensionsState,
    pub(crate) scrollbar: crate::window_scrollbar::RustWindowScrollbarState,
    pub(crate) fill_character_state: crate::window_fill_character::RustWindowFillCharacterState,
    pub flags: core::ffi::c_int,
    pub(crate) alert_queue: crate::window_alert_queue::RustWindowAlertQueueState,
    pub options: Option<crate::options::RustOptionsRef>,
    /// The sessions' links to this window, in the order they were made. A
    /// link belongs to its session, not to the window.
    pub(crate) winlinks: window_winlinks,
}

impl window {
    pub(crate) fn active_pane_id(&self) -> Option<u_int> {
        self.active_pane
            .as_ref()
            .filter(|pane| pane.is_alive())
            .map(|pane| pane.id())
    }
}

impl crate::window_dimensions::WindowDimensionsState for window {
    fn dimensions(&self) -> crate::window_dimensions::WindowDimensions {
        crate::window_dimensions::WindowDimensionsState::dimensions(&self.dimensions)
    }

    fn set_dimensions(&mut self, dimensions: crate::window_dimensions::WindowDimensions) {
        crate::window_dimensions::WindowDimensionsState::set_dimensions(
            &mut self.dimensions,
            dimensions,
        );
    }

    fn set_size(&mut self, size: crate::pane_resize::PaneSize) {
        crate::window_dimensions::WindowDimensionsState::set_size(&mut self.dimensions, size);
    }

    fn set_manual_size(&mut self, size: crate::pane_resize::PaneSize) {
        crate::window_dimensions::WindowDimensionsState::set_manual_size(
            &mut self.dimensions,
            size,
        );
    }

    fn set_pixels(&mut self, pixels: crate::window_dimensions::WindowPixelSize) {
        crate::window_dimensions::WindowDimensionsState::set_pixels(&mut self.dimensions, pixels);
    }

    fn set_pending_size(&mut self, size: crate::pane_resize::PaneSize) {
        crate::window_dimensions::WindowDimensionsState::set_pending_size(
            &mut self.dimensions,
            size,
        );
    }

    fn set_pending_pixels(&mut self, pixels: crate::window_dimensions::WindowPixelSize) {
        crate::window_dimensions::WindowDimensionsState::set_pending_pixels(
            &mut self.dimensions,
            pixels,
        );
    }

    fn set_last_new_pane(&mut self, position: crate::window_dimensions::WindowCellPosition) {
        crate::window_dimensions::WindowDimensionsState::set_last_new_pane(
            &mut self.dimensions,
            position,
        );
    }
}

impl crate::window_timestamps::WindowTimestampState for window {
    fn timestamps(&self) -> crate::window_timestamps::WindowTimestamps {
        crate::window_timestamps::WindowTimestampState::timestamps(&self.timestamps)
    }

    fn set_creation_time(&mut self, time: timeval) {
        crate::window_timestamps::WindowTimestampState::set_creation_time(
            &mut self.timestamps,
            time,
        );
    }

    fn set_activity_time(&mut self, time: timeval) {
        crate::window_timestamps::WindowTimestampState::set_activity_time(
            &mut self.timestamps,
            time,
        );
    }

    fn set_name_update_time(&mut self, time: timeval) {
        crate::window_timestamps::WindowTimestampState::set_name_update_time(
            &mut self.timestamps,
            time,
        );
    }
}

impl crate::window_fill_character::WindowFillCharacterState for window {
    fn fill_character(&self) -> Option<utf8_data> {
        crate::window_fill_character::WindowFillCharacterState::fill_character(
            &self.fill_character_state,
        )
    }

    fn set_fill_character(&mut self, character: Option<utf8_data>) {
        crate::window_fill_character::WindowFillCharacterState::set_fill_character(
            &mut self.fill_character_state,
            character,
        );
    }
}

impl crate::window_alert_queue::WindowAlertQueueState for window {
    fn alerts_are_queued(&self) -> bool {
        crate::window_alert_queue::WindowAlertQueueState::alerts_are_queued(&self.alert_queue)
    }

    fn queue_alerts(&mut self) -> bool {
        crate::window_alert_queue::WindowAlertQueueState::queue_alerts(&mut self.alert_queue)
    }

    fn clear_queued_alerts(&mut self) {
        crate::window_alert_queue::WindowAlertQueueState::clear_queued_alerts(
            &mut self.alert_queue,
        );
    }
}

impl crate::window_scrollbar::WindowScrollbarState for window {
    fn scrollbar_settings(&self) -> crate::window_scrollbar::WindowScrollbarSettings {
        crate::window_scrollbar::WindowScrollbarState::scrollbar_settings(&self.scrollbar)
    }

    fn set_scrollbar_settings(
        &mut self,
        settings: crate::window_scrollbar::WindowScrollbarSettings,
    ) {
        crate::window_scrollbar::WindowScrollbarState::set_scrollbar_settings(
            &mut self.scrollbar,
            settings,
        );
    }
}
impl crate::window_layout_selection::WindowLayoutSelectionState for window {
    fn previous_layout(&self) -> Option<core::ffi::c_int> {
        crate::window_layout_selection::WindowLayoutSelectionState::previous_layout(
            &self.layout_selection,
        )
    }

    fn remember_layout(&mut self, layout: core::ffi::c_int) {
        crate::window_layout_selection::WindowLayoutSelectionState::remember_layout(
            &mut self.layout_selection,
            layout,
        );
    }

    fn clear_previous_layout(&mut self) {
        crate::window_layout_selection::WindowLayoutSelectionState::clear_previous_layout(
            &mut self.layout_selection,
        );
    }
}

impl crate::window_saved_layout::WindowSavedLayoutState for window {
    fn saved_layout(&self) -> Option<&core::ffi::CStr> {
        crate::window_saved_layout::WindowSavedLayoutState::saved_layout(&self.saved_layout_state)
    }

    fn set_saved_layout(&mut self, layout: Option<&core::ffi::CStr>) {
        crate::window_saved_layout::WindowSavedLayoutState::set_saved_layout(
            &mut self.saved_layout_state,
            layout,
        );
    }
}

impl crate::window_name::WindowNameState for window {
    fn window_name(&self) -> Option<&core::ffi::CStr> {
        crate::window_name::WindowNameState::window_name(&self.name_state)
    }

    fn set_window_name(&mut self, name: Option<&core::ffi::CStr>) {
        crate::window_name::WindowNameState::set_window_name(&mut self.name_state, name);
    }
}

impl crate::window_trait::Window for window {
    fn window_id(&self) -> u32 {
        self.id
    }
}
impl window {
    /// The window's shared option handle.
    pub(crate) fn options_ref(&self) -> &RustOptionsRef {
        self.options.as_ref().expect("options are initialized")
    }
}
pub use crate::WindowPane;
pub use crate::window_pane::{RustWindowPane, RustWindowPaneRef, RustWindowPaneWeak, window_pane};

/// Which screen a pane is showing.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum PaneScreen {
    /// The pane's own screen, which it holds itself.
    #[default]
    Base,
    /// The screen the mode at the front of the pane's mode list draws on.
    Mode,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_mode {
    /// The mode's name, as `#{pane_mode}` and `choose-tree` report it.
    pub name: &'static core::ffi::CStr,
    pub(crate) init: window_mode_init,
    pub free: unsafe fn(&mut window_mode_entry),
    pub resize: unsafe fn(&mut window_mode_entry, u_int, u_int),
    pub optional: WindowModeOptional,
}

/// Optional mode behavior, absent unless the mode supplies it.
#[derive(Copy, Clone, Default)]
pub struct WindowModeOptional {
    /// The format the mode draws its lines with, or nothing for a mode that
    /// has none of its own.
    pub default_format: Option<&'static core::ffi::CStr>,
    pub style_changed: Option<unsafe fn(&mut window_mode_entry) -> ()>,
    pub key_table: Option<unsafe fn(&window_mode_entry) -> &'static core::ffi::CStr>,
    pub command: Option<window_mode_command>,
    pub formats: Option<unsafe fn(&window_mode_entry, &mut format_tree) -> ()>,
    pub get_screen: Option<fn(&window_mode_entry) -> Option<&RustScreen>>,
}
/// Which logical mode a pane is in, and the selector for the [`window_mode`]
/// table that mode dispatches through.
///
/// The tables live in `impl WindowMode`, in the `window_mode` module; every
/// one of them is reachable from [`WindowMode::table`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum WindowMode {
    Clock,
    Copy,
    View,
    Buffer,
    Client,
    Tree,
    Customize,
}
/// A borrowed raw-pointer view of the private state a mode's
/// [`WindowMode::init`] built.
///
/// The mode a tree belongs to, observed rather than held: the mode owns the
/// tree, so the tree names its mode this way and finds nothing once the mode
/// has gone.
#[derive(Clone, Default)]
#[repr(C)]
pub enum WindowModeData {
    #[default]
    None,
    Buffer(WindowBufferModeDataWeak),
    Client(WindowClientModeDataWeak),
    Tree(WindowTreeModeDataWeak),
    Customize(WindowCustomizeModeDataWeak),
}
/// The complete state of a window-mode entry.
///
/// The variant is the logical mode and its payload owns the mode's private
/// state. The raw-pointer [`WindowModeData`] form is derived from this value
/// when code using the translated callback interfaces needs it.
#[derive(Default)]
pub(crate) enum WindowModeState {
    #[default]
    None,
    Clock(Box<window_clock_mode_data>),
    Copy(Box<window_copy_mode_data>),
    View(Box<window_copy_mode_data>),
    Buffer(WindowBufferModeDataRef),
    Client(WindowClientModeDataRef),
    Tree(WindowTreeModeDataRef),
    Customize(WindowCustomizeModeDataRef),
}
#[derive(Clone, Default)]
#[repr(C)]
pub enum ModeTreeItemData {
    #[default]
    None,
    Buffer(Rc<window_buffer_itemdata>),
    Client(Rc<window_client_itemdata>),
    Tree(window_tree_itemdata),
    Customize(Rc<window_customize_itemdata>),
}
impl ModeTreeItemData {
    pub fn buffer(self) -> Option<Rc<window_buffer_itemdata>> {
        match self {
            ModeTreeItemData::Buffer(data) => Some(data),
            ModeTreeItemData::None => None,
            _ => panic!("not buffer-mode item data"),
        }
    }

    pub fn client(self) -> Option<Rc<window_client_itemdata>> {
        match self {
            ModeTreeItemData::Client(data) => Some(data),
            ModeTreeItemData::None => None,
            _ => panic!("not client-mode item data"),
        }
    }

    pub fn tree(&self) -> Option<window_tree_itemdata> {
        match self {
            ModeTreeItemData::Tree(data) => Some(*data),
            ModeTreeItemData::None => None,
            _ => panic!("not tree-mode item data"),
        }
    }

    pub fn customize(self) -> Option<Rc<window_customize_itemdata>> {
        match self {
            ModeTreeItemData::Customize(data) => Some(data),
            ModeTreeItemData::None => None,
            _ => panic!("not customize-mode item data"),
        }
    }
}
#[repr(C)]
pub struct window_mode_entry {
    pub(crate) pane_weak: Option<RustWindowPaneWeak>,
    pub(crate) source_pane: Option<RustWindowPaneWeak>,
    pub(crate) state: WindowModeState,
    pub(crate) screen_ready: bool,
    pub prefix: u_int,
    pub(crate) mode_tree_ref: Option<ModeTreeDataRef>,
}
impl window_mode_entry {
    /// Observes the pane while its owner still exists.
    pub(crate) fn pane_ref(&self) -> Option<RustWindowPaneWeak> {
        self.pane_weak
            .as_ref()
            .filter(|pane| pane.is_alive())
            .cloned()
    }

    /// Observes the source pane directly until it is destroyed.
    pub(crate) fn source_pane_ref(&self) -> Option<crate::window::RustWindowPaneWeak> {
        self.source_pane
            .as_ref()
            .filter(|pane| pane.is_alive())
            .cloned()
    }
}

#[derive(Default)]
#[repr(C)]
pub struct layout_cell {
    pub type_0: layout_type,
    pub flags: core::ffi::c_int,
    pub(crate) has_parent: bool,
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: core::ffi::c_int,
    pub yoff: core::ffi::c_int,
    /// The pane allocation held by this leaf, or nothing for a branch cell.
    pub wp_ref: Option<RustWindowPaneWeak>,
    pub cells: layout_cells,
}
#[derive(Default)]
#[repr(C)]
pub struct client {
    pub(crate) owner: Option<ClientWeak>,
    pub name: Option<std::ffi::CString>,
    pub peer: Option<PeerRef>,
    pub user: Option<std::ffi::CString>,
    pub queue: Option<CmdqListRef>,
    pub windows: client_windows,
    pub control_state: Option<Box<control_state>>,
    pub pause_age: u_int,
    pub pid: pid_t,
    pub fd: core::ffi::c_int,
    pub out_fd: core::ffi::c_int,
    pub event: IoHandle,
    pub retval: core::ffi::c_int,
    pub creation_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub environ: Option<Box<RustEnvironment>>,
    pub jobs: Option<Box<format_job_tree>>,
    pub title: Option<std::ffi::CString>,
    pub path: Option<std::ffi::CString>,
    pub cwd: Option<std::ffi::CString>,
    pub progress_bar: progress_bar,
    pub term_name: Option<std::ffi::CString>,
    pub term_features: core::ffi::c_int,
    pub term_type: Option<std::ffi::CString>,
    pub term_caps: Vec<std::ffi::CString>,
    pub ttyname: Option<std::ffi::CString>,
    pub tty: tty,
    pub written: size_t,
    pub discarded: size_t,
    pub redraw: size_t,
    pub repeat_timer: TimerHandle,
    pub click_timer: TimerHandle,
    pub click_loc: core::ffi::c_int,
    pub click_wp: core::ffi::c_int,
    pub click_button: u_int,
    pub click_event: mouse_event,
    pub status: status_line,
    pub theme: client_theme,
    pub input_requests: input_requests,
    pub flags: uint64_t,
    pub exit_type: client_exit_type,
    pub exit_msgtype: msgtype,
    pub exit_session: Option<std::ffi::CString>,
    pub exit_message: Option<std::ffi::CString>,
    pub(crate) keytable_ref: Option<KeyTableRef>,
    pub last_key: key_code,
    pub paste_time: time_t,
    pub redraw_panes: uint64_t,
    pub redraw_scrollbars: uint64_t,
    pub message_ignore_keys: core::ffi::c_int,
    pub message_ignore_styles: core::ffi::c_int,
    pub message_string: Option<std::ffi::CString>,
    pub(crate) message_overlay: Option<ScreenRef>,
    pub message_timer: TimerHandle,
    pub prompt_string: Option<std::ffi::CString>,
    pub(crate) prompt_overlay: Option<ScreenRef>,
    pub prompt_buffer: Vec<utf8_data>,
    pub prompt_state: cmd_find_state,
    pub prompt_last: Option<std::ffi::CString>,
    pub prompt_index: size_t,
    pub prompt: Prompt,
    pub prompt_data: PromptData,
    pub prompt_hindex: [u_int; 4],
    pub prompt_mode: client_prompt_mode,
    pub prompt_saved: Option<Vec<utf8_data>>,
    pub prompt_flags: core::ffi::c_int,
    pub prompt_type: PromptHistoryType,
    pub prompt_cursor: core::ffi::c_int,
    attached_session: Option<SessionWeak>,
    pub(crate) last_session: Option<SessionWeak>,
    pub(crate) pan_window: Option<WindowWeak>,
    pub pan_ox: u_int,
    pub pan_oy: u_int,
    /// The overlay the client is showing, what it is showing it with, and
    /// which of the two answers for the region the current drawing may
    /// cover. All four are reached from outside this module through the
    /// `overlay*` methods below: `server_client_set_overlay` and
    /// `server_client_clear_overlay` put an overlay up and take it down, and
    /// a popup retargets the region with `set_overlay_view` while it draws.
    overlay_check: OverlayCheck,
    overlay: Overlay,
    overlay_data: OverlayState,
    overlay_data_view: Option<OverlayView>,
    pub overlay_timer: TimerHandle,
    pub(crate) files: client_files_t,
    pub source_file_depth: u_int,
    pub clipboard_npanes: u_int,
}
#[repr(C)]
pub struct client_file {
    pub(crate) client_ref: Option<ClientRef>,
    pub peer: Option<PeerRef>,
    /// Which set of files this one belongs to.
    pub(crate) tree: FileOwner,
    pub stream: core::ffi::c_int,
    pub path: Option<std::ffi::CString>,
    pub buffer: Box<ByteBuffer>,
    pub event: Stream,
    pub fd: core::ffi::c_int,
    pub error: core::ffi::c_int,
    pub closed: core::ffi::c_int,
    pub done: core::ffi::c_int,
    pub(crate) cb: client_file_cb,
    pub data: ClientFileData,
}

impl client_file {
    /// The client the file belongs to, if it was opened for one.
    pub(crate) fn client(&self) -> Option<ClientRef> {
        self.client_ref.clone()
    }
}
#[derive(Clone, Default)]
#[repr(C)]
pub enum ClientFileData {
    #[default]
    None,
    LoadBuffer(Box<cmd_load_buffer_data>),
    SaveBuffer(crate::cmd::CmdqItemWeak),
    SourceFile(SourceFileRef),
    PaneInput(PaneInputRef),
}
#[derive(Clone)]
#[repr(C)]
pub struct client_window {
    pub window: u_int,
    /// The pane allocation selected by this client in the window.
    pub pane_ref: Option<RustWindowPaneWeak>,
    pub sx: u_int,
    pub sy: u_int,
}
/// A resolved command target. The default is an unresolved one: no session,
/// window or pane, and no flags.
#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_find_state {
    pub flags: core::ffi::c_int,
    /// The session the state found, observed rather than held, so that a
    /// state kept across a queue turn finds nothing rather than freed memory.
    pub(crate) s_ref: Option<SessionWeak>,
    /// The index of the link the state found, resolved against the session
    /// above, or nothing when it found none.
    pub wl_idx: Option<core::ffi::c_int>,
    /// The window the state found, observed the same way.
    pub(crate) w_ref: Option<WindowWeak>,
    /// The pane allocation the state found, independently of its saved window.
    pub wp_ref: Option<RustWindowPaneWeak>,
    pub idx: core::ffi::c_int,
}

impl cmd_find_state {
    /// The session the state found, or null when it found none or the server
    /// has since given it up.
    pub fn session(&self) -> Option<SessionRef> {
        self.s_ref.as_ref().and_then(SessionWeak::upgrade)
    }

    /// Records `s` as the session the state found.
    pub fn set_session(&mut self, s: Option<&session>) {
        let reference = s.and_then(crate::session::session_ref_of);
        self.set_session_ref(reference.as_ref());
    }

    /// Records an owner as a weak observation without borrowing its payload.
    pub(crate) fn set_session_ref(&mut self, s: Option<&SessionRef>) {
        self.s_ref = s.map(SessionRef::downgrade);
    }

    /// Retains the session owning the window link identified by this target.
    pub(crate) fn winlink_ref(&self) -> Option<crate::window::WinlinkRef> {
        crate::window::WinlinkRef::new(self.session()?, self.wl_idx?)
    }

    /// Records `wl` as the link the state found.
    pub fn set_winlink(&mut self, wl: Option<&winlink>) {
        self.wl_idx = wl.map(|wl| wl.idx);
    }

    /// The window the state found, or null the same way.
    pub fn window(&self) -> Option<WindowRef> {
        self.w_ref.as_ref().and_then(WindowWeak::upgrade)
    }

    /// Records an owner as a weak observation without borrowing its payload.
    pub(crate) fn set_window_ref(&mut self, w: Option<&WindowRef>) {
        self.w_ref = w.map(WindowRef::downgrade);
    }

    /// Observes the original pane allocation while it remains alive.
    pub(crate) fn pane_ref(&self) -> Option<RustWindowPaneWeak> {
        self.wp_ref
            .clone()
            .filter(|pane| unsafe { pane.get().is_some() })
    }

    /// Observes the original pane allocation for window-list lookup.
    pub(crate) fn pane_list_ref(&self) -> Option<RustWindowPaneWeak> {
        self.pane_ref()
    }

    /// Records the pane allocation the state found.
    pub fn set_pane(&mut self, wp: Option<&impl crate::WindowPane>) {
        self.wp_ref = wp.and_then(|pane| crate::window::window_pane_ref_of(pane));
    }
}

impl crate::CommandFindFlagsState for cmd_find_state {
    fn command_find_flags(&self) -> core::ffi::c_int {
        self.flags
    }

    fn set_command_find_flags(&mut self, flags: core::ffi::c_int) {
        self.flags = flags;
    }
}

impl crate::CommandFindIndexState for cmd_find_state {
    fn command_find_index(&self) -> core::ffi::c_int {
        self.idx
    }

    fn set_command_find_index(&mut self, index: core::ffi::c_int) {
        self.idx = index;
    }
}
/// The state of one redraw. The default is a context for no client, with
/// every offset and size zero.
#[derive(Clone, Default)]
#[repr(C)]
pub struct screen_redraw_ctx {
    pub c: Option<ClientRef>,
    pub statuslines: u_int,
    pub statustop: core::ffi::c_int,
    pub pane_status: core::ffi::c_int,
    pub pane_lines: pane_lines,
    pub pane_scrollbars: core::ffi::c_int,
    pub pane_scrollbars_pos: core::ffi::c_int,
    pub no_pane_gc: grid_cell,
    pub no_pane_gc_set: core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ox: core::ffi::c_int,
    pub oy: core::ffi::c_int,
}
#[derive(Default)]
#[repr(C)]
pub struct status_line {
    pub timer: TimerHandle,
    pub screen: RustScreen,
    /// Which screen the status line is drawn on: its own, or the overlay a
    /// message or prompt put in front of it.
    pub(crate) active: StatusActive,
    pub style: grid_cell,
    pub entries: [style_line_entry; 5],
}

/// Which screen a status line is drawing on.
#[derive(Default)]
pub(crate) enum StatusActive {
    /// The status line's own screen, which it holds itself.
    #[default]
    Own,
    /// The screen a message or prompt put in front of it, watched rather
    /// than held: the overlay slots are what keep it.
    Overlay(ScreenWeak),
}

impl status_line {
    /// Runs `use_screen` with the screen currently displayed by this status
    /// line, checking overlay borrows and keeping its owner alive for the call.
    pub(crate) fn with_active<R>(&mut self, use_screen: impl FnOnce(&mut RustScreen) -> R) -> R {
        match &self.active {
            StatusActive::Own => use_screen(&mut self.screen),
            StatusActive::Overlay(watched) => match watched.upgrade() {
                Some(held) => use_screen(&mut held.borrow_mut()),
                None => use_screen(&mut self.screen),
            },
        }
    }

    /// Replaces the active screen for an overlay redraw, handing the callback
    /// its old contents and the status line's own screen when that is distinct.
    pub(crate) fn replace_active_with<R>(
        &mut self,
        replacement: RustScreen,
        use_screen: impl FnOnce(&mut RustScreen, RustScreen, Option<&RustScreen>) -> R,
    ) -> R {
        match &self.active {
            StatusActive::Overlay(watched) => match watched.upgrade() {
                Some(held) => {
                    let mut active = held.borrow_mut();
                    let old = core::mem::replace(&mut *active, replacement);
                    use_screen(&mut active, old, Some(&self.screen))
                }
                None => {
                    let old = core::mem::replace(&mut self.screen, replacement);
                    use_screen(&mut self.screen, old, None)
                }
            },
            StatusActive::Own => {
                let old = core::mem::replace(&mut self.screen, replacement);
                use_screen(&mut self.screen, old, None)
            }
        }
    }

    /// The mode of the screen currently displayed by this status line.
    pub(crate) fn active_mode(&mut self) -> core::ffi::c_int {
        self.with_active(|screen| screen.mode())
    }

    /// Whether the status line is drawing on its own screen.
    pub(crate) fn is_own(&self) -> bool {
        matches!(self.active, StatusActive::Own)
    }
}
pub type mouse_drag_cb = Option<Rc<dyn Fn(&mut client, &mouse_event)>>;
#[repr(C)]
pub struct tty {
    /// The client whose terminal this is, observed rather than held: a tty
    /// is an inline field of its client, so holding it would be a cycle.
    pub(crate) owner: Option<ClientWeak>,
    pub start_timer: TimerHandle,
    pub clipboard_timer: TimerHandle,
    pub last_requests: time_t,
    pub sx: u_int,
    pub sy: u_int,
    pub xpixel: u_int,
    pub ypixel: u_int,
    pub cx: u_int,
    pub cy: u_int,
    pub cstyle: screen_cursor_style,
    pub ccolour: core::ffi::c_int,
    pub oflag: core::ffi::c_int,
    pub oox: u_int,
    pub ooy: u_int,
    pub osx: u_int,
    pub osy: u_int,
    pub mode: core::ffi::c_int,
    pub fg: core::ffi::c_int,
    pub bg: core::ffi::c_int,
    pub rlower: u_int,
    pub rupper: u_int,
    pub rleft: u_int,
    pub rright: u_int,
    pub event_in: IoHandle,
    pub in_0: Option<Box<ByteBuffer>>,
    pub event_out: IoHandle,
    pub out: Option<Box<ByteBuffer>>,
    pub timer: TimerHandle,
    pub discarded: size_t,
    pub tio: termios,
    pub r: VisibleRangesRef,
    pub cell: grid_cell,
    pub last_cell: grid_cell,
    pub flags: core::ffi::c_int,
    pub term: Option<crate::terminfo::TerminalRef>,
    pub mouse_last_x: u_int,
    pub mouse_last_y: u_int,
    pub mouse_last_b: u_int,
    pub mouse_drag_flag: core::ffi::c_int,
    pub mouse_scrolling_flag: core::ffi::c_int,
    pub mouse_slider_mpos: core::ffi::c_int,
    pub mouse_last_pane: core::ffi::c_int,
    pub mouse_drag_update: mouse_drag_cb,
    pub mouse_drag_release: mouse_drag_cb,
    pub key_timer: TimerHandle,
    pub key_tree: Option<Box<tty_key>>,
}
impl Default for tty {
    /// A terminal that has not been opened yet: no client, no events, no term.
    fn default() -> tty {
        tty {
            owner: None,
            start_timer: TimerHandle::default(),
            clipboard_timer: TimerHandle::default(),
            last_requests: 0,
            sx: 0,
            sy: 0,
            xpixel: 0,
            ypixel: 0,
            cx: 0,
            cy: 0,
            cstyle: 0,
            ccolour: 0,
            oflag: 0,
            oox: 0,
            ooy: 0,
            osx: 0,
            osy: 0,
            mode: 0,
            fg: 0,
            bg: 0,
            rlower: 0,
            rupper: 0,
            rleft: 0,
            rright: 0,
            event_in: IoHandle::default(),
            in_0: None,
            event_out: IoHandle::default(),
            out: None,
            timer: TimerHandle::default(),
            discarded: 0,
            tio: unsafe { core::mem::zeroed() },
            r: VisibleRangesRef::default(),
            cell: grid_cell::default(),
            last_cell: grid_cell::default(),
            flags: 0,
            term: None,
            mouse_last_x: 0,
            mouse_last_y: 0,
            mouse_last_b: 0,
            mouse_drag_flag: 0,
            mouse_scrolling_flag: 0,
            mouse_slider_mpos: 0,
            mouse_last_pane: 0,
            mouse_drag_update: None,
            mouse_drag_release: None,
            key_timer: TimerHandle::default(),
            key_tree: None,
        }
    }
}
#[repr(C)]
pub struct tty_term {
    pub(crate) name: Option<std::ffi::CString>,
    pub(crate) features: core::ffi::c_int,
    pub(crate) acs: [[u8; 2]; 256],
    pub(crate) codes: Box<[TtyCode]>,
    pub(crate) flags: core::ffi::c_int,
    pub(crate) registration: Option<crate::terminfo::TerminalRegistration>,
}
pub type winlinks = std::collections::BTreeMap<core::ffi::c_int, Box<winlink>>;
/// The modes a pane has open, the one it is showing first. A mode belongs to
/// the pane until `window_pane_reset_mode` takes it off.
pub type window_modes = Vec<Box<window_mode_entry>>;
/// The links to one window, in the order they were made.
/// The sessions' links to a window, in the order they were made, named by
/// the session that holds each and the index it holds it at. A link belongs
/// to its session, not to the window.
pub(crate) type window_winlinks = Vec<(SessionWeak, core::ffi::c_int)>;

/// A session's most-recently-used window links, most recent first.
/// The indexes of the links on a session's most-recent order, newest first.
pub type winlink_stack = Vec<core::ffi::c_int>;
/// A window's sole pane owners, in pane-index order. Moving an owner between
/// windows preserves the pane's address. Server removal tears down resources
/// and frees the payload; outstanding observations do not delay destruction.
pub(crate) type window_panes_t = Vec<RustWindowPaneRef>;
/// An ordered pile of panes — the most-recently-used stack, most recent
/// first, and the stacking order, topmost first. A plain Rust vector rather
/// than a TAILQ so that dropping the window tears it down without writing
/// through per-pane back-pointers.
/// The ids of the panes on one of a window's two orders, most recent or
/// topmost first. A pane is named by its id, so an order never holds one the
/// window has given up.
pub type window_pane_stack_t = Vec<RustWindowPaneWeak>;
/// The cells directly under one layout cell, left to right or top to bottom.
/// A cell belongs to the list it hangs in.
pub type layout_cells = Vec<Box<layout_cell>>;
pub(crate) type client_files_t = std::collections::BTreeMap<core::ffi::c_int, ClientFileRef>;

/// Which set of files a file belongs to: one client's, the whole process's,
/// another owned set, or none once it has been removed.
#[derive(Clone, Default)]
pub(crate) enum FileOwner {
    #[default]
    None,
    /// The files of one client, observed rather than held.
    Client(ClientWeak),
    /// The process-global set, borrowed only while visiting it.
    Global(&'static crate::tree::GlobalTree<core::ffi::c_int, ClientFileRef>),
    /// An owned set observed without keeping its files in an ownership cycle.
    #[cfg(test)]
    Shared(Weak<RefCell<client_files_t>>),
}

impl FileOwner {
    /// Visits the file set and returns its client owner alongside the result.
    /// The file-set borrow ends before the caller can invoke a file callback.
    ///
    /// # Safety
    /// For client-owned sets, the caller must exclude other access to the file
    /// set during the visit, including access from reentrant callbacks.
    pub(crate) unsafe fn with_tree<R>(
        &self,
        visit: impl FnOnce(&mut client_files_t) -> R,
    ) -> (Option<ClientRef>, Option<R>) {
        match self {
            FileOwner::None => (None, None),
            FileOwner::Client(watched) => match watched.upgrade() {
                Some(c) => {
                    let result = visit(unsafe { &mut (*c.0.value.as_ptr()).files });
                    (Some(c), Some(result))
                }
                None => (None, None),
            },
            FileOwner::Global(files) => (None, Some(visit(&mut files.map()))),
            #[cfg(test)]
            FileOwner::Shared(watched) => match watched.upgrade() {
                Some(files) => {
                    let result = visit(&mut files.borrow_mut());
                    (None, Some(result))
                }
                None => (None, None),
            },
        }
    }
}
pub type client_windows = std::collections::BTreeMap<u_int, client_window>;
pub(crate) enum ClientFileEvent<'a> {
    Read {
        client: Option<&'a ClientRef>,
        error: core::ffi::c_int,
        buffer: &'a mut ByteBuffer,
        data: &'a ClientFileData,
    },
    Done {
        client: Option<ClientRef>,
        path: std::ffi::CString,
        error: core::ffi::c_int,
        buffer: ByteBuffer,
        data: ClientFileData,
    },
    CheckExit,
}
pub(crate) type client_file_cb = Option<Rc<dyn for<'a> Fn(ClientFileEvent<'a>)>>;
#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum OverlayData {
    None,
    Menu(MenuDataRef),
    Popup(PopupDataRef),
    DisplayPanes(DisplayPanesRef),
}

#[derive(Default)]
#[repr(C)]
pub enum OverlayState {
    #[default]
    None,
    Menu(MenuDataRef),
    Popup(PopupDataRef),
    DisplayPanes(DisplayPanesRef),
}

impl OverlayState {
    fn view_data(&mut self, view: Option<OverlayView>) -> OverlayData {
        match view {
            Some(OverlayView::Menu) => {
                let OverlayState::Popup(popup) = self else {
                    panic!("a menu view belongs to a popup");
                };
                let popup = popup.borrow();
                let menu = popup.md.as_ref().expect("a menu view has a menu").clone();
                OverlayData::Menu(menu)
            }
            Some(OverlayView::Nothing) => OverlayData::None,
            None | Some(OverlayView::Popup) => self.data(),
        }
    }

    pub fn data(&mut self) -> OverlayData {
        match self {
            OverlayState::None => OverlayData::None,
            OverlayState::Menu(data) => OverlayData::Menu(data.clone()),
            OverlayState::Popup(data) => OverlayData::Popup(data.clone()),
            OverlayState::DisplayPanes(data) => OverlayData::DisplayPanes(data.clone()),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, OverlayState::None)
    }

    pub fn menu(&mut self) -> MenuDataRef {
        self.data().menu()
    }

    pub fn popup(&mut self) -> PopupDataRef {
        self.data().popup()
    }

    pub fn display_panes(&mut self) -> DisplayPanesRef {
        self.data().display_panes()
    }
}

impl client {
    /// Retains the observed attachment while its session is still alive.
    pub(crate) fn attached_session(&self) -> Option<SessionRef> {
        self.attached_session
            .as_ref()
            .and_then(SessionWeak::upgrade)
    }

    /// Records the attachment without running session-change callbacks or redraws.
    pub(crate) fn set_attached_session(&mut self, session: Option<&SessionRef>) {
        self.attached_session = session.map(SessionRef::downgrade);
    }

    /// The peer handle carrying the connected client's messages.
    pub(crate) fn peer_handle(&self) -> &PeerRef {
        self.peer.as_ref().expect("a connected client has a peer")
    }

    /// The environment the client was started with, or null for a client that
    /// carries none.
    pub(crate) fn environ_mut(&mut self) -> &mut RustEnvironment {
        self.environ
            .as_deref_mut()
            .expect("a client is created with its environment")
    }

    /// The key table the client's next key is looked up in, if one is assigned.
    pub(crate) fn keytable(&self) -> Option<KeyTableRef> {
        self.keytable_ref.clone()
    }

    pub(crate) fn current_overlay_data(&mut self) -> OverlayData {
        self.overlay_data.view_data(self.overlay_data_view)
    }

    /// The overlay the client is showing, which is `Overlay::None` when it is
    /// showing none.
    pub(crate) fn overlay(&self) -> Overlay {
        self.overlay
    }

    /// Which overlay answers for the region the current drawing may cover.
    pub(crate) fn overlay_check(&self) -> OverlayCheck {
        self.overlay_check
    }

    /// What the overlay is showing, which the module that put it up owns.
    pub(crate) fn overlay_data(&mut self) -> &mut OverlayState {
        &mut self.overlay_data
    }

    /// Puts `overlay` up, showing `data`. The region goes to whichever
    /// overlay answers for it, and any retargeting the last one left behind
    /// is dropped.
    pub(crate) fn set_overlay(&mut self, overlay: Overlay, data: OverlayState) {
        self.overlay_check = overlay.check();
        self.overlay = overlay;
        self.overlay_data = data;
        self.overlay_data_view = None;
    }

    /// Takes the overlay down and hands back what it was showing, so that the
    /// caller can free it. Nothing answers for the region afterwards.
    pub(crate) fn take_overlay(&mut self) -> (Overlay, OverlayState) {
        let overlay = core::mem::replace(&mut self.overlay, Overlay::None);
        let data = core::mem::take(&mut self.overlay_data);
        self.overlay_check = OverlayCheck::None;
        self.overlay_data_view = None;
        (overlay, data)
    }

    /// Points the region at `view` for the drawing that follows, which is how
    /// a popup hands it to the menu it carries and takes it back afterwards.
    pub(crate) fn set_overlay_view(&mut self, view: OverlayView) {
        match view {
            OverlayView::Menu => {
                self.overlay_check = OverlayCheck::Menu;
                self.overlay_data_view = Some(OverlayView::Menu);
            }
            OverlayView::Nothing => {
                self.overlay_check = OverlayCheck::None;
                self.overlay_data_view = Some(OverlayView::Nothing);
            }
            OverlayView::Popup => {
                self.overlay_check = OverlayCheck::Popup;
                self.overlay_data_view = None;
            }
        }
    }

    /// Gives the drawing data back to the overlay itself, which is what a
    /// popup does once the menu it was carrying is gone.
    pub(crate) fn clear_overlay_view(&mut self) {
        self.overlay_data_view = None;
    }
}

/// Who answers for the region the current drawing may cover, while a popup
/// points it somewhere other than itself.
#[derive(Copy, Clone)]
pub(crate) enum OverlayView {
    /// The menu the popup carries, which owns the drawing data as well.
    Menu,
    /// Nobody, so no drawing may be covered and there is nothing to draw
    /// with, which is the state a popup writes its own screen under.
    Nothing,
    /// The popup itself, drawing with what the overlay was put up with.
    Popup,
}
/// Which overlay a client is showing, and so which module owns
/// `client::overlay_data`.
///
/// A client carries one overlay at a time, installed by
/// `server_client_set_overlay` and taken down by
/// `server_client_clear_overlay`; the three that exist are named here rather
/// than reached through a table of callbacks, so that asking which one is up
/// — as `popup_present` does — is a comparison of values.
///
/// `None` is the zero byte, because a client arrives from `xcalloc`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum Overlay {
    #[default]
    None = 0,
    Menu,
    Popup,
    /// `display-panes`, which reads keys unless `-N` was given.
    DisplayPanes {
        keys: bool,
    },
}

/// Which overlay decides what the current drawing may cover.
///
/// This follows the overlay for as long as one is up, but a popup retargets
/// it while it draws or feeds its own pane: the menu it carries answers for
/// the region then, and nothing answers while the popup writes its own
/// screen.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum OverlayCheck {
    #[default]
    None = 0,
    Menu,
    Popup,
}

impl Overlay {
    pub fn is_some(self) -> bool {
        self != Overlay::None
    }
    pub fn is_none(self) -> bool {
        self == Overlay::None
    }
}

impl OverlayCheck {
    pub fn is_some(self) -> bool {
        self != OverlayCheck::None
    }
    pub fn is_none(self) -> bool {
        self == OverlayCheck::None
    }
}
/// Who put the prompt on the status line, and so who reads what is typed
/// into it and owns `client::prompt_data`.
///
/// A client shows one prompt at a time, opened by `status_prompt_set`. The
/// variants are the callers that open one: naming them keeps the answer to
/// "is this still my prompt?" — which `cmd_command_prompt` asks before it
/// continues its command queue — a comparison of values.
///
/// A client keeps the last value after `status_prompt_clear`, exactly as it
/// kept the last callback pointer; `client::prompt_string` says whether a
/// prompt is up.
///
/// `None` is the zero byte, because a client arrives from `xcalloc`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum Prompt {
    #[default]
    None = 0,
    CommandPrompt,
    ConfirmBefore,
    ModeTreeSearch,
    ModeTreeFilter,
    WindowTreeCommand,
    WindowTreeKillCurrent,
    WindowTreeKillTagged,
    CustomizeSetOption,
    CustomizeSetCommand,
    CustomizeSetNote,
    CustomizeChangeCurrent,
    CustomizeChangeTagged,
    /// A prompt opened by a unit test, which records what it is answered.
    #[cfg(test)]
    Recorder,
}

impl Prompt {
    pub fn is_some(self) -> bool {
        self != Prompt::None
    }
    pub fn is_none(self) -> bool {
        self == Prompt::None
    }
}

/// The state owned by the prompt currently displayed on a client.
///
/// Prompt-private records are boxed directly. Mode prompts instead box a weak
/// handle to their shared mode state, so closing the mode drops that state and
/// the prompt's later input finds nothing to act on. A customize prompt boxes
/// its own item, which carries that weak handle in `prompt_owner`.
#[derive(Default)]
#[repr(C)]
pub enum PromptData {
    #[default]
    None,
    CommandPrompt(Box<cmd_command_prompt_cdata>),
    ConfirmBefore(Box<cmd_confirm_before_data>),
    ModeTree(Box<ModeTreeDataWeak>),
    WindowTree(Box<WindowTreeModeDataWeak>),
    CustomizeSet(Box<window_customize_itemdata>),
    CustomizeChange(Box<WindowCustomizeModeDataWeak>),
}

impl PartialEq for PromptData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::CommandPrompt(left), Self::CommandPrompt(right)) => {
                core::ptr::eq(&**left, &**right)
            }
            (Self::ConfirmBefore(left), Self::ConfirmBefore(right)) => {
                core::ptr::eq(&**left, &**right)
            }
            (Self::ModeTree(left), Self::ModeTree(right)) => left.ptr_eq(right),
            (Self::WindowTree(left), Self::WindowTree(right)) => left.ptr_eq(right),
            (Self::CustomizeSet(left), Self::CustomizeSet(right)) => {
                core::ptr::eq(&**left, &**right)
            }
            (Self::CustomizeChange(left), Self::CustomizeChange(right)) => left.ptr_eq(right),
            _ => false,
        }
    }
}

impl Eq for PromptData {}

impl ::core::fmt::Debug for PromptData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::None => "None",
            Self::CommandPrompt(_) => "CommandPrompt(..)",
            Self::ConfirmBefore(_) => "ConfirmBefore(..)",
            Self::ModeTree(_) => "ModeTree(..)",
            Self::WindowTree(_) => "WindowTree(..)",
            Self::CustomizeSet(_) => "CustomizeSet(..)",
            Self::CustomizeChange(_) => "CustomizeChange(..)",
        })
    }
}

/// Every client the server holds, in the order they connected. A client
/// belongs to the client registry, not to this list.
pub(crate) type clients_t = Vec<ClientRef>;
pub(crate) type sessions_t = std::collections::BTreeMap<std::ffi::CString, SessionRef>;
/// Where a command being parsed came from. The default names no source file,
/// no issuing item or client, and no target.
#[derive(Clone, Default)]
#[repr(C)]
pub struct cmd_parse_input {
    pub flags: core::ffi::c_int,
    /// The file the command line was read from, if it was read from one.
    pub file: Option<std::ffi::CString>,
    pub line: u_int,
    pub item: Option<crate::cmd::CmdqItemWeak>,
    /// The client the parse is for, observed rather than held.
    pub(crate) c: Option<ClientWeak>,
    pub fs: cmd_find_state,
}

impl cmd_parse_input {
    /// The file the command line came from, or nothing when it came from
    /// none.
    pub fn file(&self) -> Option<&core::ffi::CStr> {
        self.file.as_deref()
    }

    /// The client the parse is for, or nothing when it is for none.
    pub fn client(&self) -> Option<ClientRef> {
        self.c.as_ref().and_then(ClientWeak::upgrade)
    }
}
/// What one spawn was asked for. Strings borrow the command that asked,
/// and the session owner remains alive through the operation.
#[derive(Default)]
#[repr(C)]
pub struct spawn_context<'a> {
    pub item: Option<crate::cmd::CmdqItemWeak>,
    pub s: Option<SessionRef>,
    pub wl_idx: Option<core::ffi::c_int>,
    pub tc: Option<ClientWeak>,
    pub wp0: Option<RustWindowPaneWeak>,
    pub name: Option<&'a core::ffi::CStr>,
    pub argv: Vec<std::ffi::CString>,
    pub environ: Option<Box<RustEnvironment>>,
    pub idx: core::ffi::c_int,
    pub cwd: Option<&'a core::ffi::CStr>,
    pub flags: core::ffi::c_int,
}
#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub enum TtyCtxArg {
    #[default]
    None,
    Pane(RustWindowPaneWeak),
    Popup(PopupDataWeak),
}
#[derive(Clone, Default)]
#[repr(C)]
pub struct tty_ctx<'a> {
    pub redraw_cb: tty_ctx_redraw_cb,
    pub set_client_cb: tty_ctx_set_client_cb,
    pub arg: TtyCtxArg,
    pub cell: Option<grid_cell>,
    pub flags: core::ffi::c_int,
    pub value: TtyCtxValue<'a>,
    pub ocx: u_int,
    pub ocy: u_int,
    pub orupper: u_int,
    pub orlower: u_int,
    pub xoff: core::ffi::c_int,
    pub yoff: core::ffi::c_int,
    pub rxoff: core::ffi::c_int,
    pub ryoff: core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub bg: u_int,
    pub defaults: grid_cell,
    pub palette: Option<colour_palette>,
    pub wox: u_int,
    pub woy: u_int,
    pub wsx: u_int,
    pub wsy: u_int,
}
#[derive(Copy, Clone)]
pub struct tty_ctx_data<'a> {
    pub data: &'a [u8],
}
pub type tty_ctx_redraw_cb = Option<unsafe fn(&tty_ctx) -> ()>;
#[derive(Copy, Clone)]
pub struct tty_ctx_sel<'a> {
    pub clip: &'a core::ffi::CStr,
    pub data: &'a [u8],
}
pub type tty_ctx_set_client_cb = Option<unsafe fn(&mut tty_ctx, &mut client) -> core::ffi::c_int>;
/// What a terminal command carries besides its position: a count, a run of
/// bytes, or a selection to hand to the terminal.
#[derive(Copy, Clone, Default)]
pub enum TtyCtxValue<'a> {
    #[default]
    None,
    Num(u_int),
    Data(tty_ctx_data<'a>),
    Sel(tty_ctx_sel<'a>),
}
/// An argument's kind and payload.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub enum ArgsValue {
    #[default]
    None,
    String(std::ffi::CString),
    Commands {
        cmdlist: Option<CmdListRef>,
        /// The printed form of `cmdlist`, which a caller asking for the
        /// value as a string wants and which is the same every time it is
        /// asked for. Printing it needs no more than the value itself, so
        /// it is filled in on the first ask rather than by whoever built it.
        cached: std::cell::OnceCell<std::ffi::CString>,
    },
}

impl ArgsValue {
    /// The string the value holds, which every caller of this has already
    /// established it is.
    pub fn string(&self) -> &core::ffi::CStr {
        match self {
            ArgsValue::String(string) => string,
            _ => panic!("not a string argument"),
        }
    }
}

#[derive(Default)]
#[repr(C)]
pub struct ibuf {
    pub buf: bytes::BytesMut,
    pub size: size_t,
    pub max: size_t,
    pub wpos: size_t,
    pub rpos: size_t,
    pub fd: core::ffi::c_int,
    /// Whether the bytes were copied out of somebody else's range rather
    /// than allocated here. A borrowed buffer may be read, but not grown,
    /// resized, given a descriptor, or handed to a queue.
    pub borrowed: bool,
}
#[derive(Default)]
#[repr(C)]
pub struct imsg {
    pub hdr: imsg_hdr,
    /// The original body range, independent of the buffer's read cursor.
    pub(crate) body_range: core::ops::Range<usize>,
    /// The buffer the message was read out of, which the message owns until
    /// it is given up or handed on to a queue.
    pub buf: Option<Box<ibuf>>,
}
#[repr(C)]
pub struct args_entry {
    pub flag: u_char,
    pub values: Vec<ArgsValue>,
    pub count: u_int,
    pub flags: core::ffi::c_int,
}
pub type args_tree = std::collections::BTreeMap<u_char, Box<args_entry>>;
/// The children of one tree item, or the tree's own top level, in the order
/// they were added. An item belongs to the list it sits on.
pub type mode_tree_list = Vec<Box<mode_tree_item>>;

/// Whether `haystack` holds `needle` anywhere in it. An empty needle is in
/// everything, which is what `strstr` answers for one.
pub fn bytes_have(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty() || haystack.windows(needle.len()).any(|at| at == needle)
}

/// [`bytes_have`] ignoring case. The C folds by the locale and hmux runs in
/// the C locale, where that is the ASCII fold this does.
pub fn bytes_have_nocase(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || haystack
            .windows(needle.len())
            .any(|at| at.eq_ignore_ascii_case(needle))
}

/// [`bytes_have`] over C strings, which is `strstr` asked as the question
/// every caller of it was really asking: none of them wants the position it
/// returns, only whether it is there.
pub fn cstr_has(haystack: &core::ffi::CStr, needle: &core::ffi::CStr) -> bool {
    bytes_have(haystack.to_bytes(), needle.to_bytes())
}

/// [`cstr_has`] ignoring case, which is `strcasestr`.
pub fn cstr_has_nocase(haystack: &core::ffi::CStr, needle: &core::ffi::CStr) -> bool {
    bytes_have_nocase(haystack.to_bytes(), needle.to_bytes())
}
use crate::screen::RustScreen;

impl VisibleRangesRef {
    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }
    pub fn ensure_capacity(&self, count: u_int) {
        self.borrow_mut().ensure_capacity(count)
    }
    pub fn set_overlay_range(
        &self,
        x: u_int,
        y: u_int,
        width: u_int,
        height: u_int,
        px: u_int,
        py: u_int,
        count: u_int,
    ) {
        self.borrow_mut()
            .set_overlay_range(x, y, width, height, px, py, count)
    }
    pub unsafe fn set_visible_ranges(
        &self,
        pane: Option<&impl crate::WindowPane>,
        x: core::ffi::c_int,
        y: core::ffi::c_int,
        width: u_int,
    ) {
        unsafe { self.borrow_mut().set_visible_ranges(pane, x, y, width) }
    }
    pub unsafe fn clip_visible_ranges(
        &self,
        pane: Option<&impl crate::WindowPane>,
        x: core::ffi::c_int,
        y: core::ffi::c_int,
        width: u_int,
    ) {
        unsafe { self.borrow_mut().clip_visible_ranges(pane, x, y, width) }
    }
}

#[cfg(test)]
mod reference_borrow_tests {
    use super::*;
    use crate::tests::test_fixtures::{globals, zeroed, zeroed_client};

    #[test]
    fn peer_snapshot_releases_client_borrow_and_keeps_peer_alive() {
        let _guard = globals();
        let client = zeroed_client();
        let peer = PeerRef::new(*zeroed::<crate::proc::tmuxpeer>());
        peer.borrow_mut().uid = 42;
        client.0.value.borrow_mut().peer = Some(peer);
        let retained = client.peer_handle();
        client.0.value.borrow_mut().peer = None;
        drop(client);
        assert_eq!(retained.uid(), 42);
    }

    #[test]
    #[should_panic(expected = "already mutably borrowed")]
    fn client_snapshot_checks_borrows_across_owners() {
        let _guard = globals();
        let client = zeroed_client();
        let other = client.clone();
        let _exclusive = client.0.value.borrow_mut();
        other.pid();
    }
}

impl ClientRef {
    /// Records a command exit code without exposing mutable client storage.
    pub(crate) fn set_return_code(&self, code: core::ffi::c_int) {
        self.0.value.borrow_mut().retval = code;
    }

    /// Enters a source-file invocation with the existing wrapping depth arithmetic.
    pub(crate) fn enter_source_file(&self) {
        let mut client = self.0.value.borrow_mut();
        client.source_file_depth = client.source_file_depth.wrapping_add(1);
    }

    /// Completes a source-file invocation with the existing wrapping depth arithmetic.
    pub(crate) fn leave_source_file(&self) {
        let mut client = self.0.value.borrow_mut();
        client.source_file_depth = client.source_file_depth.wrapping_sub(1);
    }
}

impl ClientRef {
    /// Reports whether a prompt string is installed, including an empty string.
    pub(crate) fn has_prompt_text(&self) -> bool {
        self.0.value.borrow().prompt_string.is_some()
    }

    /// Logs a wait-channel observation using the existing client pointer label.
    /// The pointer is formatted here and is not exposed to the command.
    pub(crate) fn log_wait_channel(&self, name: &core::ffi::CStr, woken: bool) {
        {
            crate::log::log_debug(
                if woken {
                    c"wait channel %s already woken (%p)"
                } else {
                    c"wait channel %s not woken (%p)"
                },
                crate::fmt_args![name, self.as_ptr()],
            );
        }
    }
}
