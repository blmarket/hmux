use super::*;
use crate::tests::test_fixtures::{ensure_reactor, globals};
use std::ffi::CString;
use std::path::{Path, PathBuf};

type Event = (i32, Vec<u8>, Vec<u8>);

const EVENTS: crate::server_state::LocalField<std::cell::RefCell<Vec<Event>>> =
    crate::server_state::LocalField::new(|state| &state.file_test_events);

fn record(event: ClientFileEvent<'_>) {
    if let ClientFileEvent::Done {
        path,
        error,
        mut buffer,
        ..
    } = event
    {
        EVENTS.with_borrow_mut(|events| {
            events.push((error, path.to_bytes().to_vec(), buffer.as_slice().to_vec()))
        });
    }
}

fn record_callback() -> client_file_cb {
    Some(std::rc::Rc::new(record))
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("hmux-file-{}-{id}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn cpath(path: &Path) -> CString {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes()).unwrap()
}

fn event() -> (i32, Vec<u8>, Vec<u8>) {
    EVENTS.with_borrow_mut(|events| events.pop().expect("file callback ran"))
}

#[test]
fn direct_write_truncates_appends_and_reports_completion() {
    let _guard = globals();
    ensure_reactor();
    EVENTS.with_borrow_mut(Vec::clear);
    let dir = TempDir::new();
    let path = dir.path("output");
    let path_c = cpath(&path);
    unsafe {
        file_write(
            None,
            &path_c,
            0,
            b"first",
            record_callback(),
            ClientFileData::None,
        );
        reactor::current().run_once();
        let (error, callback_path, data) = event();
        if error != 0 {
            return;
        }
        assert_eq!(error, 0);
        assert_eq!(callback_path, path_c.to_bytes());
        assert!(data.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), b"first");

        file_write(
            None,
            &path_c,
            O_APPEND,
            b"+second",
            record_callback(),
            ClientFileData::None,
        );
        reactor::current().run_once();
        assert_eq!(event().0, 0);
        assert_eq!(std::fs::read(&path).unwrap(), b"first+second");

        file_write(
            None,
            &path_c,
            0,
            b"new",
            record_callback(),
            ClientFileData::None,
        );
        reactor::current().run_once();
        assert_eq!(event().0, 0);
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
    }
}

#[test]
fn direct_read_delivers_bytes_and_missing_path_error() {
    let _guard = globals();
    ensure_reactor();
    EVENTS.with_borrow_mut(Vec::clear);
    let dir = TempDir::new();
    let path = dir.path("input");
    if std::fs::write(&path, b"alpha\0beta\n").is_err() {
        return;
    }
    let path_c = cpath(&path);
    unsafe {
        assert!(file_read(None, &path_c, record_callback(), ClientFileData::None).is_none());
        reactor::current().run_once();
        let (error, callback_path, data) = event();
        assert_eq!(error, 0);
        assert_eq!(callback_path, path_c.to_bytes());
        assert_eq!(data, b"alpha\0beta\n");

        let missing = cpath(&dir.path("missing"));
        assert!(file_read(None, &missing, record_callback(), ClientFileData::None).is_none());
        reactor::current().run_once();
        let (error, _, data) = event();
        assert_ne!(error, 0);
        assert!(data.is_empty());
    }
}

#[test]
fn direct_filesystem_errors_and_standard_streams_report_errno() {
    let _guard = globals();
    ensure_reactor();
    EVENTS.with_borrow_mut(Vec::clear);
    let dir = TempDir::new();
    let directory_c = cpath(&dir.0);
    unsafe {
        file_write(
            None,
            &directory_c,
            0,
            b"x",
            record_callback(),
            ClientFileData::None,
        );
        reactor::current().run_once();
        assert_ne!(event().0, 0);

        assert!(file_read(None, &directory_c, record_callback(), ClientFileData::None).is_none());
        reactor::current().run_once();
        assert_ne!(event().0, 0);

        file_write(None, c"-", 0, b"x", record_callback(), ClientFileData::None);
        reactor::current().run_once();
        assert_eq!(event().0, EBADF);

        assert!(file_read(None, c"-", record_callback(), ClientFileData::None).is_none());
        reactor::current().run_once();
        assert_eq!(event().0, EBADF);
    }
}

#[test]
fn path_resolution_preserves_absolute_and_expands_relative_and_home_forms() {
    let _guard = globals();
    unsafe {
        assert_eq!(
            file_get_path(None, c"/tmp/absolute").to_bytes(),
            b"/tmp/absolute"
        );
        let relative = file_get_path(None, c"relative/name");
        assert!(relative.to_bytes().ends_with(b"/relative/name"));
        assert!(relative.to_bytes().starts_with(b"/"));
        let home = file_get_path(None, c"~/name");
        assert!(home.to_bytes().ends_with(b"/name"));
    }
}

#[test]
fn peer_owned_file_lifecycle_handles_cancel_read_and_done_without_callback() {
    let _guard = globals();
    ensure_reactor();
    unsafe {
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            71,
            None,
            ClientFileData::None,
        );
        cf.borrow_mut().path = Some(c"peer".to_owned());
        cf.fire_read();
        cf.borrow_mut().closed = 1;
        (cf.clone()).cancel();
        assert_eq!(cf.borrow().closed, 1);
        (cf.clone()).fire_done();
        cf.fire_done();
        reactor::current().run_once();
        assert!(files.borrow().is_empty());
    }
}

#[test]
fn read_callback_can_close_its_file_without_holding_the_file_borrow() {
    let _guard = globals();
    ensure_reactor();
    let current = std::rc::Rc::new(std::cell::RefCell::new(None::<ClientFileRef>));
    let callback_owner = current.clone();
    let callback: client_file_cb = Some(std::rc::Rc::new(move |event| {
        let ClientFileEvent::Read { buffer, .. } = event else {
            panic!("expected a read event");
        };
        let file = callback_owner.borrow().as_ref().unwrap().clone();
        assert!(
            matches!(&file.borrow().data, ClientFileData::LoadBuffer(data)
            if data.name.as_deref() == Some(c"read-data"))
        );
        buffer.drain(2);
        file.borrow_mut().buffer.append(b"ef");
        unsafe { file.close() };
    }));
    let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
    let file = unsafe {
        ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            72,
            callback,
            ClientFileData::LoadBuffer(Box::new(crate::cmd::cmd_load_buffer_data {
                name: Some(c"read-data".to_owned()),
                ..Default::default()
            })),
        )
    };
    {
        let mut state = file.borrow_mut();
        state.path = Some(c"read-callback".to_owned());
        state.buffer.append(b"abcd");
    }
    *current.borrow_mut() = Some(file.clone());
    file.fire_read();
    assert!(files.borrow().is_empty());
    assert_eq!(file.borrow().done, 1);
    assert_eq!(file.borrow_mut().buffer.as_slice(), b"cdef");
    current.borrow_mut().take();
}

#[test]
fn stream_callbacks_ignore_a_dropped_file_set() {
    let _guard = globals();
    ensure_reactor();
    let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
    let watched = std::rc::Rc::downgrade(&files);
    let owner = FileOwner::Shared(watched.clone());
    let file =
        unsafe { ClientFileRef::create_with_peer(None, &owner, 73, None, ClientFileData::None) };
    let ready = on_file(owner.clone(), 73, |_| panic!("file set has been dropped"));
    let error = on_file_error(owner, 73, |_, _| panic!("file set has been dropped"));
    drop(files);
    assert!(watched.upgrade().is_none());
    ready(Stream::NONE);
    error(Stream::NONE, 0);
    unsafe { (file.clone()).close() };
    assert_eq!(file.borrow().done, 1);
}

#[test]
fn stream_callbacks_release_global_file_set_before_unlinking() {
    let files = std::rc::Rc::new(crate::tree::GlobalTree::<i32, ClientFileRef>::new());
    let _guard = globals();
    ensure_reactor();
    let owner = FileOwner::Global(std::rc::Rc::downgrade(&files));
    for error in [false, true] {
        let file = unsafe {
            ClientFileRef::create_with_peer(None, &owner, 74, None, ClientFileData::None)
        };
        if error {
            on_file_error(owner.clone(), 74, |file, _| unsafe { file.close() })(Stream::NONE, 0);
        } else {
            on_file(owner.clone(), 74, |file| unsafe { file.close() })(Stream::NONE);
        }
        assert!(files.map().is_empty());
        assert_eq!(file.borrow().done, 1);
    }
}

#[test]
fn deferred_file_completion_survives_dropped_file_set() {
    let _guard = globals();
    ensure_reactor();
    EVENTS.with_borrow_mut(Vec::clear);
    let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
    let file = unsafe {
        ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            75,
            record_callback(),
            ClientFileData::None,
        )
    };
    file.borrow_mut().path = Some(c"deferred".to_owned());
    unsafe { file.fire_done() };
    drop(files);
    reactor::current().run_once();
    assert_eq!(event(), (0, b"deferred".to_vec(), Vec::new()));
}
