//! Real Window construction for integration tests, with explicit option cleanup.
use hmux2::src::options::{options_create, options_default, options_free, options_set_number};
use hmux2::src::shared::options::options;
use hmux2::src::shared::window::WindowRef;
use hmux2::src::tmux::global_w_options;

pub struct WindowOptions {
    owner: Box<options>,
    previous: *mut options,
}

impl WindowOptions {
    pub unsafe fn new() -> Self {
        let mut owner = options_create(None);
        for name in [
            c"pane-scrollbars",
            c"pane-scrollbars-position",
            c"pane-border-status",
            c"monitor-silence",
            c"monitor-activity",
        ] {
            let definition = hmux2::src::options_table::options_table
                .iter()
                .find(|entry| entry.name == Some(name))
                .unwrap();
            options_default(&mut *owner, definition);
        }
        options_set_number(&mut *owner, c"pane-scrollbars".as_ptr(), 0);
        options_set_number(&mut *owner, c"pane-border-status".as_ptr(), 0);
        let previous = std::mem::replace(&mut global_w_options, &mut *owner);
        Self { owner, previous }
    }

    pub unsafe fn create(&self, width: u32, height: u32) -> WindowRef {
        hmux2::src::window::window_create(width, height, 0, 0)
    }

    /// Every created Window must have been explicitly released first.
    pub unsafe fn free(self) {
        global_w_options = self.previous;
        options_free(self.owner);
        hmux2::src::reactor::shutdown_runtime();
    }
}
