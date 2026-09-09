use super::*;
use crate::compat::{
    imsg_free, imsg_get, imsg_get_type, imsgbuf_allow_fdpass, imsgbuf_clear, imsgbuf_flush,
    imsgbuf_init, imsgbuf_queuelen, imsgbuf_read,
};
use crate::ffi::socketpair;
use crate::reactor::{Interest, IoWatch, WatchMode};
use crate::tests::test_fixtures::{ensure_reactor, globals, zeroed};
use core::ffi::{c_int, c_short, c_void};
use core::ptr::null_mut;

unsafe fn never(_fd: c_int, _events: c_short, _arg: *mut c_void) {}

struct IdentifyPeer {
    process: ProcessRef,
    peer: PeerRef,
    far: Box<imsgbuf>,
    fds: [c_int; 2],
}

impl IdentifyPeer {
    fn new() -> Self {
        ensure_reactor();
        let mut fds = [-1; 2];
        unsafe {
            assert_eq!(
                socketpair(AF_UNIX, SOCK_STREAM as c_int, 0, fds.as_mut_ptr()),
                0
            );
            let mut value = Self {
                process: ProcessRef::default(),
                peer: PeerRef::new(*zeroed::<tmuxpeer>()),
                far: zeroed::<imsgbuf>(),
                fds,
            };
            assert_eq!(imsgbuf_init(&mut value.peer.borrow_mut().ibuf, fds[0]), 0);
            imsgbuf_allow_fdpass(&mut value.peer.borrow_mut().ibuf);
            assert_eq!(imsgbuf_init(&mut value.far, fds[1]), 0);
            value.peer.borrow_mut().event.set_callback(
                fds[0],
                Interest::Read,
                WatchMode::Once,
                move |fd, events| never(fd, events, null_mut()),
            );
            client_proc = Some(value.process.clone());
            client_peer = Some(value.peer.clone());
            value
        }
    }

    unsafe fn messages(&mut self) -> Vec<uint32_t> {
        unsafe {
            while imsgbuf_queuelen(&mut self.peer.borrow_mut().ibuf) > 0 {
                assert_eq!(imsgbuf_flush(&mut self.peer.borrow_mut().ibuf), 0);
                assert_eq!(imsgbuf_read(&mut self.far), 1);
            }
            let mut types = Vec::new();
            loop {
                let Ok(Some((mut message, _len))) = imsg_get(&mut self.far) else {
                    break;
                };
                types.push(imsg_get_type(&message));
                imsg_free(message);
            }
            types
        }
    }
}

impl Drop for IdentifyPeer {
    fn drop(&mut self) {
        unsafe {
            client_proc = None;
            client_peer = None;
            self.peer.borrow_mut().event.disable();
            imsgbuf_clear(&mut self.peer.borrow_mut().ibuf);
            imsgbuf_clear(&mut self.far);
            close(self.fds[0]);
            close(self.fds[1]);
        }
    }
}

#[test]
fn identify_sends_fixed_fields_capabilities_descriptors_environment_and_done() {
    let _guard = globals();
    let mut peer = IdentifyPeer::new();
    unsafe {
        client_flags = CLIENT_CONTROL as u64;
        let caps = vec![c"RGB".to_owned(), c"clipboard".to_owned()];
        client_send_identify(c"/dev/pts/focused", c"xterm-256color", &caps, c"/tmp", 7);
        let messages = peer.messages();
        assert_eq!(
            messages
                .iter()
                .filter(|&&ty| ty == MSG_IDENTIFY_LONGFLAGS)
                .count(),
            2
        );
        for ty in [
            MSG_IDENTIFY_TERM,
            MSG_IDENTIFY_FEATURES,
            MSG_IDENTIFY_TTYNAME,
            MSG_IDENTIFY_CWD,
            MSG_IDENTIFY_STDIN,
        ] {
            assert!(messages.contains(&ty), "missing {ty} from {messages:?}");
        }
        assert_eq!(
            messages
                .iter()
                .filter(|&&ty| ty == MSG_IDENTIFY_TERMINFO)
                .count(),
            2
        );
        client_flags = 0;
    }
}
