use hmux2::src::cfg::{cfg_client, cfg_finished};
use hmux2::src::server_client::server_client_get_cwd;
use hmux2::src::shared::{client::client, session::session};
use std::rc::{Rc, Weak};

#[test]
fn startup_observer_and_owned_directory_preserve_lifetime_and_precedence() {
    unsafe {
        let startup = client::new();
        (*startup.get()).cwd = Some(c"/startup".to_owned());
        cfg_client = Rc::downgrade(&startup);
        cfg_finished = 0;
        let other = client::new();
        (*other.get()).cwd = Some(c"/other".to_owned());
        let session = session::new();
        (*session.get()).cwd = Some(c"/session".to_owned());
        let saved = server_client_get_cwd(Some(&*other.get()), Some(&session)).unwrap();
        assert_eq!(saved.as_c_str(), c"/startup");
        (*startup.get()).cwd = None;
        assert!(server_client_get_cwd(Some(&*other.get()), None).is_none());
        drop(startup);
        assert!((&cfg_client).upgrade().is_none());
        assert_eq!(saved.as_c_str(), c"/startup");
        assert_eq!(
            server_client_get_cwd(Some(&*other.get()), Some(&session)).unwrap().as_c_str(),
            c"/other",
        );
        assert_eq!(
            server_client_get_cwd(None, Some(&session)).unwrap().as_c_str(),
            c"/session",
        );
        cfg_client = Weak::new();
        cfg_finished = 1;
    }
}
