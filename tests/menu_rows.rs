use hmux2::src::environ::{environ_create, environ_free};
use hmux2::src::menu::{menu_add_item, menu_create, menu_free};
use hmux2::src::options::{options_create, options_free};
use hmux2::src::shared::client::client;
use hmux2::src::shared::key::KEYC_NONE;
use hmux2::src::shared::menu::menu_item;
use hmux2::src::tmux::{global_environ, global_options, global_s_options, global_w_options};
use std::ffi::{CStr, CString};
use std::ptr::{null, null_mut};

#[test]
fn runtime_rows_own_expansions_across_growth_and_keep_separators() {
    unsafe {
        let previous = (
            global_options,
            global_s_options,
            global_w_options,
            global_environ,
        );
        global_options = options_create(null_mut());
        global_s_options = options_create(null_mut());
        global_w_options = options_create(null_mut());
        global_environ = environ_create();
        let owner = client::new();
        let client = hmux2::src::shared::rc::as_ptr(&owner);
        (*client).tty.sx = 120;
        let menu = menu_create(c"Rows".as_ptr());
        let separator = menu_item {
            name: c"".as_ptr(),
            key: KEYC_NONE,
            command: null(),
        };
        menu_add_item(menu, &separator, null_mut(), client, null_mut());
        assert_eq!((*menu).count, 0);

        for index in 0..64 {
            let name = CString::new(format!("Row {index}")).unwrap();
            let command = CString::new(format!("display-message {index}")).unwrap();
            let definition = menu_item {
                name: name.as_ptr(),
                key: KEYC_NONE,
                command: command.as_ptr(),
            };
            menu_add_item(menu, &definition, null_mut(), client, null_mut());
            // Definitions expire on each iteration; runtime strings must survive.
        }
        let suppressed = menu_item {
            name: c"#{?0,shown,}".as_ptr(),
            key: KEYC_NONE,
            command: null(),
        };
        menu_add_item(menu, &suppressed, null_mut(), client, null_mut());
        assert_eq!((*menu).count, 64);
        menu_add_item(menu, &separator, null_mut(), client, null_mut());
        menu_add_item(menu, &separator, null_mut(), client, null_mut());
        assert_eq!((*menu).count, 65);
        for (index, row) in (&(*menu).items)[..64].iter().enumerate() {
            assert_eq!(
                CStr::from_ptr(row.name_ptr()).to_bytes(),
                format!("Row {index}").as_bytes()
            );
            assert_eq!(
                row.command.as_ref().unwrap().to_bytes(),
                format!("display-message {index}").as_bytes()
            );
        }
        assert!((&(*menu).items)[64].name.is_none());
        assert!((&(*menu).items)[64].command.is_none());
        menu_free(menu);
        hmux2::src::reactor::event_loop();
        assert_eq!(std::rc::Rc::strong_count(&owner), 1);
        options_free(global_options);
        options_free(global_s_options);
        options_free(global_w_options);
        environ_free(global_environ);
        (
            global_options,
            global_s_options,
            global_w_options,
            global_environ,
        ) = previous;
    }
}
