//! Real Window construction for integration tests, with explicit option cleanup.
use hmux2::src::options::{options_create, options_default, options_free, options_set_number};
use hmux2::src::shared::options::options;
use hmux2::src::shared::window::WindowRef;
use hmux2::src::tmux::{global_options, global_w_options};
use hmux2::src::window::Window as _;

pub struct WindowOptions {
    owner: Box<options>,
    previous: *mut options,
    server_owner: Box<options>,
    previous_server: *mut options,
}

impl WindowOptions {
    pub unsafe fn new() -> Self {
        let mut owner = options_create(None);
        for definition in hmux2::src::options_table::options_table
            .iter()
            .filter(|entry| {
                entry.scope
                    & (hmux2::src::shared::options::OPTIONS_TABLE_WINDOW
                        | hmux2::src::shared::options::OPTIONS_TABLE_PANE)
                    != 0
            })
        {
            options_default(&mut *owner, definition);
        }
        options_set_number(&mut *owner, c"pane-scrollbars".as_ptr(), 0);
        options_set_number(&mut *owner, c"pane-border-status".as_ptr(), 0);
        let mut server_owner = options_create(None);
        let definition = hmux2::src::options_table::options_table
            .iter()
            .find(|entry| entry.name == Some(c"extended-keys"))
            .unwrap();
        options_default(&mut *server_owner, definition);
        let previous_server = std::mem::replace(&mut global_options, &mut *server_owner);
        let previous = std::mem::replace(&mut global_w_options, &mut *owner);
        Self {
            owner,
            previous,
            server_owner,
            previous_server,
        }
    }

    pub unsafe fn create(&self, width: u32, height: u32) -> WindowRef {
        hmux2::src::shared::window::WindowRef::create(width, height, 0, 0)
    }

    /// Every created Window must have been explicitly released first.
    pub unsafe fn free(self) {
        global_w_options = self.previous;
        global_options = self.previous_server;
        options_free(self.owner);
        options_free(self.server_owner);
        hmux2::src::reactor::shutdown_runtime();
    }
}
