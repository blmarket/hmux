use hmux2::src::environ::environ_create;
use hmux2::src::menu::{menu_add_item, menu_add_items, menu_create};
use hmux2::src::options::{options_create, options_free};
use hmux2::src::server_client::Client as _;
use hmux2::src::shared::client::client;
use hmux2::src::shared::key::KEYC_NONE;
use hmux2::src::shared::menu::menu_item;
use hmux2::src::tmux::{global_environ, global_options, global_s_options, global_w_options};
use std::ffi::CString;
use std::ptr::null_mut;

#[test]
fn runtime_rows_own_expansions_across_growth_and_keep_separators() {
    unsafe {
        let previous = (
            global_options,
            global_s_options,
            global_w_options,
            global_environ.take(),
        );
        let mut global_options_owner = options_create(None);
        global_options = &raw mut *global_options_owner;
        let mut global_s_options_owner = options_create(None);
        global_s_options = &raw mut *global_s_options_owner;
        let mut global_w_options_owner = options_create(None);
        global_w_options = &raw mut *global_w_options_owner;
        global_environ = Some(environ_create());
        let owner = client::new();
        owner.borrow_terminal_mut().sx = 120;
        let mut menu = menu_create(c"Rows");
        let separator = menu_item {
            name: c"",
            key: KEYC_NONE,
            command: None,
        };
        menu_add_item(&mut *menu, None, None, Some(&owner), null_mut());
        menu_add_items(&mut *menu, &[], Some(&owner));
        menu_add_item(&mut *menu, Some(&separator), None, Some(&owner), null_mut());
        assert_eq!((*menu).count(), 0);

        for index in 0..64 {
            let name = CString::new(format!("Row {index}")).unwrap();
            let command = CString::new(format!("display-message {index}")).unwrap();
            let definition = menu_item {
                name: &name,
                key: KEYC_NONE,
                command: Some(&command),
            };
            menu_add_item(
                &mut *menu,
                Some(&definition),
                None,
                Some(&owner),
                null_mut(),
            );
            // Definitions expire on each iteration; runtime strings must survive.
        }
        let suppressed = menu_item {
            name: c"#{?0,shown,}",
            key: KEYC_NONE,
            command: None,
        };
        menu_add_item(
            &mut *menu,
            Some(&suppressed),
            None,
            Some(&owner),
            null_mut(),
        );
        assert_eq!((*menu).count(), 64);
        menu_add_item(&mut *menu, Some(&separator), None, Some(&owner), null_mut());
        menu_add_item(&mut *menu, Some(&separator), None, Some(&owner), null_mut());
        assert_eq!((*menu).count(), 65);
        for (index, row) in (&(*menu).items)[..64].iter().enumerate() {
            assert_eq!(
                row.name.as_ref().unwrap().as_bytes(),
                format!("Row {index}").as_bytes()
            );
            assert_eq!(
                row.command.as_ref().unwrap().to_bytes(),
                format!("display-message {index}").as_bytes()
            );
        }
        assert!((&(*menu).items)[64].name.is_none());
        assert!((&(*menu).items)[64].command.is_none());
        {
            let names = [
                CString::new("借用").unwrap(),
                CString::new("Last row").unwrap(),
            ];
            let definitions = [
                menu_item {
                    name: &names[0],
                    key: KEYC_NONE,
                    command: None,
                },
                menu_item {
                    name: &names[1],
                    key: KEYC_NONE,
                    command: Some(c""),
                },
            ];
            // A bounded stack array needs no terminal item, and its text may expire.
            menu_add_items(&mut *menu, &definitions, Some(&owner));
        }
        assert_eq!((*menu).count(), 67);
        assert_eq!((&(*menu).items)[65].name.as_deref(), Some(c"借用"));
        assert!((&(*menu).items)[65].command.is_none());
        assert_eq!((&(*menu).items)[66].name.as_deref(), Some(c"Last row"));
        assert_eq!((&(*menu).items)[66].command.as_deref(), Some(c""));
        drop(menu);
        hmux2::src::reactor::event_loop();
        assert_eq!(std::rc::Rc::strong_count(&owner), 1);
        options_free(global_options_owner);
        options_free(global_s_options_owner);
        options_free(global_w_options_owner);
        drop(global_environ.take());
        (
            global_options,
            global_s_options,
            global_w_options,
            global_environ,
        ) = previous;
    }
}
