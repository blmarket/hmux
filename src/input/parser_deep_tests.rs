use super::*;
use crate::WindowPane;
use crate::pane_identity::PaneIdentity;
use crate::style::{ColourEngine, RustColourEngine};
use crate::tests::test_fixtures::{Pane, Window, ensure_reactor, globals};

struct Parser {
    _window: Window,
    pane: Pane,
    ictx: InputCtxRef,
    _guard: crate::tests::test_fixtures::GlobalsGuard,
}

impl Parser {
    fn new() -> Self {
        let guard = globals();
        ensure_reactor();
        let mut window = Window::new(481, "parser-deep", 80, 24);
        let mut pane = Pane::new(482, 80, 24, 100);
        window.add_pane(&mut pane);
        let wp = pane.ptr();
        let ictx = unsafe {
            RustColourEngine.init_palette((*wp).palette_mut());
            *(*wp).ictx_mut() = Some(InputCtxRef::create(
                InputOwner::Pane((*wp).pane_id()),
                Stream::NONE,
            ));
            ictx_opt((*wp).ictx()).unwrap()
        };
        Self {
            _window: window,
            pane,
            ictx,
            _guard: guard,
        }
    }

    unsafe fn parse(&mut self, bytes: &'static [u8]) {
        unsafe {
            input_parse_buffer(
                &mut *self.pane.ptr(),
                ByteBuffer::from(bytes::Bytes::from_static(bytes)),
            );
        }
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe {
            let wp = self.pane.ptr();
            if let Some(ictx) = (*wp).ictx_mut().take() {
                ictx.close();
            }
            RustColourEngine.free_palette(Some((*wp).palette_mut()));
        }
    }
}

#[test]
fn device_queries_reports_and_window_operations_return_to_ground() {
    let mut p = Parser::new();
    unsafe {
        p.parse(b"\x1b[c\x1b[0c\x1b[>c\x1b[>0c");
        p.parse(b"\x1b[5n\x1b[6n\x1b[?6n\x1b[?15n\x1b[?25n\x1b[?26n");
        p.parse(b"\x1b[14t\x1b[16t\x1b[18t\x1b[19t\x1b[20t\x1b[21t");
        p.parse(b"\x1b[22;0t\x1b[22;1t\x1b[22;2t\x1b[23;0t\x1b[23;1t\x1b[23;2t");
        p.parse(b"\x1b[8;30;100t\x1b[10;2t\x1b[11t\x1b[13t\x1b[99t");
        assert_eq!(p.ictx.borrow().state.name, c"ground");
    }
}

#[test]
fn private_modes_cover_mouse_focus_sync_origin_and_alternate_screen() {
    let mut p = Parser::new();
    unsafe {
        p.parse(b"\x1b[?3;5;9;12;25;47;1000;1002;1003;1004;1005;1006;1047;1049;2004;2026;2031h");
        p.parse(b"\x1b[?3;5;9;12;25;47;1000;1002;1003;1004;1005;1006;1047;1049;2004;2026;2031l");
        p.parse(b"\x1b[?1;3;6;7;12;25;47;1000;1002;1003;1004;1005;1006;1047;1049;2004;2026;2031$p");
        p.parse(b"\x1b[?9999h\x1b[?9999l\x1b[9999h\x1b[9999l");
        assert_eq!(p.ictx.borrow().state.name, c"ground");
    }
}

#[test]
fn dcs_status_requests_cover_known_unknown_cancelled_and_fragmented_forms() {
    let mut p = Parser::new();
    unsafe {
        p.parse(b"\x1bP$qm\x1b\\\x1bP$qr\x1b\\\x1bP$q q\x1b\\");
        p.parse(b"\x1bP$q\"q\x1b\\\x1bP$q\"p\x1b\\\x1bP$qz\x1b\\");
        p.parse(b"\x1bP1;2;3+qpayload\x1b\\\x1bPignored\x18");
        p.parse(b"\x1bP$q");
        assert_eq!(p.ictx.borrow().state.name, c"dcs_handler");
        p.parse(b"m\x1b\\");
        assert_eq!(p.ictx.borrow().state.name, c"ground");
    }
}

#[test]
fn osc_variants_cover_progress_markers_palette_resets_and_invalid_values() {
    let mut p = Parser::new();
    unsafe {
        p.parse(b"\x1b]7;file://host/tmp/path\x07\x1b]7;bad\x07");
        p.parse(
            b"\x1b]9;4;0\x07\x1b]9;4;1;25\x07\x1b]9;4;2;50\x07\x1b]9;4;3;75\x07\x1b]9;4;4;100\x07",
        );
        p.parse(b"\x1b]133;A\x07\x1b]133;C\x07\x1b]133;D;7\x07\x1b]133;Z\x07");
        p.parse(b"\x1b]4;0;?\x07\x1b]4;999;red\x07\x1b]4;2;invalid\x07");
        p.parse(b"\x1b]104\x07\x1b]104;1;2;3\x07\x1b]110\x07\x1b]111\x07\x1b]112\x07");
        p.parse(b"\x1b]8;id=x:foo=bar;https://example.invalid\x1b\\x\x1b]8;;\x1b\\");
        p.parse(b"\x1b]999;ignored\x07\x1b]0;cancelled\x18");
        assert_eq!(p.ictx.borrow().state.name, c"ground");
    }
}

#[test]
fn csi_edge_parameters_cover_erase_tabs_regions_repeat_and_graphics_modes() {
    let mut p = Parser::new();
    unsafe {
        p.parse(b"abcdef\x1b[0b\x1b[2b\x1b[-1b\x1b[999b");
        p.parse(b"\x1b[0J\x1b[1J\x1b[2J\x1b[3J\x1b[4J\x1b[0K\x1b[1K\x1b[2K\x1b[4K");
        p.parse(b"\x1b[0g\x1b[3g\x1b[1g\x1b[999g\x1b[2Z\x1b[999Z");
        p.parse(b"\x1b[1;24r\x1b[24;1r\x1b[0;0r\x1b[5;5r\x1b[999;1000r");
        p.parse(b"\x1b[1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;16;17;18;19;20;21;22;23;24m");
        p.parse(b"\x1b[38;5m\x1b[38;5;999m\x1b[38;2;1;2m\x1b[48:2:0:1:2:3:4m");
        p.parse(b"\x1b[1;2;3;4;5;6;7;8;9;10S\x1b[1;2;3;4;5T");
        assert_eq!(p.ictx.borrow().state.name, c"ground");
    }
}

#[test]
fn escape_and_string_cancellation_cover_intermediate_and_utf8_paths() {
    let mut p = Parser::new();
    unsafe {
        p.parse(b"\x1bc\x1bN\x1bO\x1bV\x1bW\x1bX\x1bZ\x1b\\");
        p.parse(b"\x1b(0\x1b(A\x1b(B\x1b)0\x1b)A\x1b)B\x1b*0\x1b+0");
        p.parse(b"\x1b_abc\x18\x1b^abc\x1a\x1bPabc\x18\x1b]abc\x1a");
        p.parse(b"utf8:\xc2\xa3 \xe2\x98\x83 bad:\xc2A \xf0\x9f\x98\x80");
        p.parse(b"\x7f\x00\x01\x02\x03\x04\x05\x06\x0b\x0c");
        p.parse(b"\x1b\\");
        assert_eq!(p.ictx.borrow().state.name, c"ground");
    }
}

#[test]
fn palette_access_observes_immediate_pane_destruction() {
    let _guard = globals();
    ensure_reactor();
    let pane = RustWindowPaneRef::new(Box::new(window_pane::default()));
    let observer = pane.downgrade();
    let parser = unsafe { InputCtxRef::create(InputOwner::Pane(pane.id()), Stream::NONE) };
    unsafe {
        parser
            .borrow_mut()
            .with_palette_mut(|palette| palette.unwrap().fg = 42);
        assert_eq!(pane.get().unwrap().palette().fg, 42);
    }
    drop(pane);
    assert!(observer.upgrade().is_none());
    assert!(parser.borrow().pane_ref().is_none());
    unsafe {
        parser
            .borrow_mut()
            .with_palette_mut(|palette| assert!(palette.is_none()));
        parser.close();
    }
}

#[test]
fn parser_window_observation_follows_moves_and_preserves_transfer_context() {
    let parser = Parser::new();
    let original = parser._window.reference();
    let destination = Window::new(483, "destination", 80, 24);
    let destination = destination.reference();
    unsafe {
        let id = parser.ictx.borrow().pane_ref().unwrap().id();
        parser
            .ictx
            .borrow()
            .pane_ref()
            .unwrap()
            .get()
            .unwrap()
            .options_ref()
            .set_number(c"allow-rename", 1);
        let pane = crate::window::window_panes_take(
            &mut original.as_window_mut(),
            &crate::window::window_pane_find_by_id(id).expect("the pane exists"),
        )
        .unwrap();
        assert!(pane.window().is_none());
        assert!(parser.ictx.borrow().window_ref().unwrap().ptr_eq(&original));
        parser.ictx.borrow_mut().input_buf = b"renamed-original\0".to_vec();
        input_exit_rename(&mut parser.ictx.borrow_mut());
        assert_eq!(original.window_name().as_deref(), Some(c"renamed-original"));
        crate::window::window_panes_insert_tail(&mut destination.as_window_mut(), pane);
        assert!(
            parser
                .ictx
                .borrow()
                .window_ref()
                .unwrap()
                .ptr_eq(&destination)
        );
        parser.ictx.borrow_mut().input_buf = b"renamed-destination\0".to_vec();
        input_exit_rename(&mut parser.ictx.borrow_mut());
        assert_eq!(
            destination.window_name().as_deref(),
            Some(c"renamed-destination")
        );
        let pane = crate::window::window_panes_take(
            &mut destination.as_window_mut(),
            &crate::window::window_pane_find_by_id(id).expect("the pane exists"),
        )
        .unwrap();
        assert!(pane.window().is_none());
        assert!(
            parser
                .ictx
                .borrow()
                .window_ref()
                .unwrap()
                .ptr_eq(&destination)
        );
        crate::window::window_panes_insert_tail(&mut original.as_window_mut(), pane);
        assert!(parser.ictx.borrow().window_ref().unwrap().ptr_eq(&original));
    }
}
