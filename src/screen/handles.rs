use super::{RustScreen, Screen};
use crate::grid::RustGrid;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// A borrowed screen capability that cannot replace its owner's screen.
///
/// ```compile_fail
/// use tmux_c2rs::{WindowPane, RustScreen, Screen};
/// fn replace(pane: &mut dyn WindowPane) {
///     *pane.base_mut() = RustScreen::new(80, 24, 100);
/// }
/// ```
///
/// ```compile_fail
/// use tmux_c2rs::WindowPane;
/// fn recover(pane: &mut dyn WindowPane) {
///     let _ = pane.base_mut().into_inner();
/// }
/// ```
pub struct ScreenMut<'a>(&'a mut RustScreen);

impl<'a> ScreenMut<'a> {
    pub(crate) fn new(screen: &'a mut RustScreen) -> Self {
        Self(screen)
    }
    pub(super) fn as_inner(&mut self) -> &mut RustScreen {
        self.0
    }
    pub(super) fn into_inner(self) -> &'a mut RustScreen {
        self.0
    }
}

impl<'a> core::ops::Deref for ScreenMut<'a> {
    type Target = dyn Screen<Grid = RustGrid> + 'a;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[cfg(test)]
impl ScreenMut<'_> {
    pub(crate) fn write_test_cell(&mut self, x: u32, y: u32, cell: &crate::types::grid_cell) {
        self.0.write_test_cell(x, y, cell);
    }
    pub(crate) fn scroll_test_history(&mut self, bg: u32) {
        self.0.scroll_test_history(bg);
    }
    pub(crate) fn clear_path(&mut self) {
        self.0.clear_path();
    }
}

impl core::ops::DerefMut for ScreenMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
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

pub(super) struct ScreenWriteLease<'a> {
    target: &'a ScreenRef,
}

impl<'a> ScreenWriteLease<'a> {
    pub(super) fn borrow(&self) -> std::cell::Ref<'a, RustScreen> {
        self.target.0.screen.borrow()
    }

    pub(super) fn borrow_mut(&self) -> std::cell::RefMut<'a, RustScreen> {
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

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, dyn Screen<Grid = RustGrid>> {
        assert!(!self.0.writing.get(), "screen has an active writer");
        std::cell::RefMut::map(self.0.screen.borrow_mut(), |screen| {
            screen as &mut dyn Screen<Grid = RustGrid>
        })
    }

    pub(super) fn begin_write(&self) -> ScreenWriteLease<'_> {
        assert!(!self.0.writing.replace(true), "screen has an active writer");
        ScreenWriteLease { target: self }
    }

    pub(crate) fn take_for_pane(&self) -> RustScreen {
        assert!(!self.0.writing.get(), "screen has an active writer");
        core::mem::replace(
            &mut *self.0.screen.borrow_mut(),
            RustScreen::new_with_server_options(1, 1, 0),
        )
    }

    pub(crate) fn redraw<R>(
        &self,
        replacement: RustScreen,
        draw: impl FnOnce(ScreenMut<'_>, RustScreen) -> R,
    ) -> R {
        assert!(!self.0.writing.get(), "screen has an active writer");
        let mut screen = self.0.screen.borrow_mut();
        let old = core::mem::replace(&mut *screen, replacement);
        draw(ScreenMut::new(&mut screen), old)
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
