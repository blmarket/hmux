"""Run current layout operations against one RefCell for the whole Window.

The layout source and cell implementation are extracted on every run. Model
adapters and rendering/resize callbacks are small stubs: the test checks borrow
boundaries and copied geometry, not the application's complete lifecycle.
"""
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest

from model_boundary_inventory import mask

ROOT = Path(__file__).resolve().parents[1]
OPERATIONS = (
    "layout_create_cell", "layout_set_size", "layout_make_leaf", "layout_make_node",
    "layout_cell_is_tiled", "layout_cell_has_tiled_child", "layout_cell_is_first_tiled",
    "layout_cell_is_last_tiled", "layout_cell_is_top", "layout_cell_is_bottom",
    "layout_add_horizontal_border", "layout_fix_panes", "layout_clamp_floating_panes",
    "layout_clamp_floating_cell", "layout_resize_pane_to", "layout_resize_floating_pane_to",
    "layout_resize_floating_pane", "layout_assign_pane",
)


def extract_item(source, name, kind="fn"):
    masked = mask(source)
    pattern = r"^(?:pub(?:\([^)]*\))? )?(?:unsafe )?" + kind + " " + re.escape(name) + r"\b"
    matches = list(re.finditer(pattern, masked, re.MULTILINE))
    if len(matches) != 1:
        raise ValueError(f"layout/core.rs: expected exactly one {kind} {name}, found {len(matches)}")
    match = matches[0]
    opening = masked.find("{", match.end())
    if opening < 0:
        raise ValueError(f"layout/core.rs: {name} has no body")
    depth, end = 1, opening + 1
    while depth and end < len(masked):
        depth += (masked[end] == "{") - (masked[end] == "}")
        end += 1
    if depth:
        raise ValueError(f"layout/core.rs: {name} has an unclosed body")
    return source[match.start():end]


def current_source_fixture():
    core = (ROOT / "src/layout/core.rs").read_text()
    operations = "\n".join(extract_item(core, name) for name in OPERATIONS)
    tests = "#[cfg(test)]\n" + extract_item(core, "floating_clamp_tests", "mod")
    cell = (ROOT / "src/shared/layout.rs").read_text()
    return (FIXTURE.replace("// SOURCE_CELL", cell)
            .replace("// SOURCE_OPERATIONS", operations)
            .replace("// SOURCE_TESTS", tests))


class WindowLayoutBorrowTests(unittest.TestCase):
    def test_extraction_requires_current_unique_complete_bodies(self):
        for source, message in [("", "found 0"), ("fn f() {}\nfn f() {}", "found 2"),
                                ("fn f() {", "unclosed")]:
            with self.assertRaisesRegex(ValueError, message):
                extract_item(source, "f")
        source = 'fn f() { let s = "}"; /* { */ body(); }\nfn next() {}'
        self.assertEqual(extract_item(source, "f"), source.split("\n")[0])

    def test_actual_operations_with_whole_window_refcell(self):
        rustc = shutil.which("rustc")
        self.assertIsNotNone(rustc, "rustc is required for the layout borrow regression")
        with tempfile.TemporaryDirectory(prefix="hmux-layout-borrows-") as directory:
            directory = Path(directory)
            source = directory / "layout.rs"
            binary = directory / "layout-tests"
            source.write_text(current_source_fixture())
            subprocess.run([rustc, "--edition=2021", "--test", str(source), "-o", str(binary)], check=True)
            subprocess.run([str(binary)], check=True)


FIXTURE = r'''
#![allow(dead_code, unused, non_camel_case_types, private_interfaces)]
use std::{cell::{RefCell, Ref, RefMut, UnsafeCell}, ffi::CStr, rc::{Rc, Weak}};
mod shared {
    pub mod abi { pub type u_int = u32; }
    pub mod pane {
        use super::super::*;
        pub struct window_pane {
            pub layout_cell: Option<LayoutCellId>,
            pub observer: Weak<UnsafeCell<Self>>,
            pub window: Weak<RefCell<window>>,
            pub scrollbar_style: ScrollbarStyle,
            pub sx: u32, pub sy: u32, pub xoff: i32, pub yoff: i32, pub flags: i32,
            pub border: i32, pub lines: u32, pub reserve: bool,
            pub on_resize: Option<Box<dyn FnOnce(&WindowRef)>>,
        }
        impl window_pane {
            pub fn new() -> Rc<UnsafeCell<Self>> {
                Rc::new_cyclic(|observer| UnsafeCell::new(Self {
                    layout_cell: None, observer: observer.clone(), window: Weak::new(),
                    scrollbar_style: ScrollbarStyle { width: 1, pad: 0 },
                    sx: 0, sy: 0, xoff: 0, yoff: 0, flags: 0,
                    border: 0, lines: PANE_LINES_NONE, reserve: false, on_resize: None,
                }))
            }
        }
    }
    pub mod layout {
// SOURCE_CELL
    }
}
mod src { pub mod window {
    #[derive(Clone, Copy)] pub enum LayoutView { Visible }
    #[derive(Clone, Copy)] pub enum PaneOrder { Stacking }
}}
use shared::{abi::*, layout::*, pane::window_pane};
type WindowRef = Rc<RefCell<window>>;
type PaneRef = Rc<UnsafeCell<window_pane>>;
const PANE_MINIMUM: i32 = 1;
const PANE_MAXIMUM: i32 = 10000;
const PANE_STATUS_TOP: i32 = 1;
const PANE_STATUS_BOTTOM: i32 = 2;
const PANE_SCROLLBARS_LEFT: i32 = 0;
const PANE_REDRAWSCROLLBAR: i32 = 1;
struct ScrollbarStyle { width: i32, pad: i32 }
struct Scrollbars { position: i32 }
struct Geometry { size: (u32,u32), offset: (i32,i32), floating: bool, top_border: bool, bottom_border: bool }
fn fatalx(f: impl FnOnce(&mut dyn std::io::Write)->std::io::Result<()>) -> ! {
    let mut out = Vec::new(); f(&mut out).unwrap(); panic!("{}", String::from_utf8_lossy(&out));
}
struct window {
    root: Option<Box<layout_cell>>, saved: Option<Box<layout_cell>>,
    panes: Vec<Weak<UnsafeCell<window_pane>>>, generation: u64,
    resize_changes: Vec<i32>, resized: Vec<(u32,u32)>,
}
impl window {
    fn new() -> WindowRef { Rc::new(RefCell::new(Self {
        root: None, saved: None, panes: vec![], generation: 0, resize_changes: vec![], resized: vec![],
    })) }
    fn find(&self,id:LayoutCellId)->Option<&layout_cell> {
        self.root.as_deref().and_then(|root|root.find(id))
            .or_else(||self.saved.as_deref().and_then(|root|root.find(id)))
    }
    fn find_mut(&mut self,id:LayoutCellId)->Option<&mut layout_cell> {
        self.root.as_deref_mut().and_then(|root|root.find_mut(id))
            .or_else(||self.saved.as_deref_mut().and_then(|root|root.find_mut(id)))
    }
}
trait Window {
    fn next_pane(&self,after:Option<&PaneRef>)->Option<PaneRef>;
    fn step_pane(&self,order:src::window::PaneOrder,after:Option<&Weak<UnsafeCell<window_pane>>>,reverse:bool)->Option<PaneRef>;
    fn pane_layout_geometry(&self,pane:&Weak<UnsafeCell<window_pane>>,view:src::window::LayoutView)->Option<Geometry>;
    fn scrollbars(&self)->Scrollbars;
    fn invalidate_scene(&self);
    fn borrow_layout_cell(&self,id:LayoutCellId)->Option<Ref<'_,layout_cell>>;
    fn borrow_layout_cell_mut(&self,id:LayoutCellId)->Option<RefMut<'_,layout_cell>>;
    fn release(self,from:&CStr);
}
impl Window for WindowRef {
    fn next_pane(&self,after:Option<&PaneRef>)->Option<PaneRef> {
        self.step_pane(src::window::PaneOrder::Stacking,after.map(Rc::downgrade).as_ref(),false)
    }
    fn step_pane(&self,_:src::window::PaneOrder,after:Option<&Weak<UnsafeCell<window_pane>>>,reverse:bool)->Option<PaneRef> {
        assert!(!reverse);
        let state=self.borrow();
        let index=if let Some(after)=after {state.panes.iter().position(|p|p.ptr_eq(after))?+1} else {0};
        state.panes.get(index).and_then(Weak::upgrade)
    }
    fn pane_layout_geometry(&self,pane:&Weak<UnsafeCell<window_pane>>,_:src::window::LayoutView)->Option<Geometry> {
        fn find<'a>(cell:&'a layout_cell,pane:&Weak<UnsafeCell<window_pane>>)->Option<&'a layout_cell> {
            if cell.wp.ptr_eq(pane) {Some(cell)} else {cell.cells.iter().find_map(|child|find(child,pane))}
        }
        let state=self.borrow(); let root=state.root.as_deref()?; let cell=find(root,pane)?;
        let root_ptr=root as *const layout_cell as *mut layout_cell;
        let cell_ptr=cell as *const layout_cell as *mut layout_cell;
        Some(Geometry { size:(cell.g.sx,cell.g.sy), offset:(cell.g.xoff,cell.g.yoff),
            floating:cell.flags & LAYOUT_CELL_FLOATING != 0,
            top_border:unsafe {layout_add_horizontal_border(root_ptr,cell_ptr,PANE_STATUS_TOP)!=0},
            bottom_border:unsafe {layout_add_horizontal_border(root_ptr,cell_ptr,PANE_STATUS_BOTTOM)!=0} })
    }
    fn scrollbars(&self)->Scrollbars {let _loan=self.borrow(); Scrollbars {position:PANE_SCROLLBARS_LEFT}}
    fn invalidate_scene(&self) {self.borrow_mut().generation+=1;}
    fn borrow_layout_cell(&self,id:LayoutCellId)->Option<Ref<'_,layout_cell>> {
        Ref::filter_map(self.borrow(),|state|state.find(id)).ok()
    }
    fn borrow_layout_cell_mut(&self,id:LayoutCellId)->Option<RefMut<'_,layout_cell>> {
        RefMut::filter_map(self.borrow_mut(),|state|state.find_mut(id)).ok()
    }
    fn release(self,_:&CStr) {let _loan=self.borrow_mut();}
}
trait Pane {
    unsafe fn geometry(&self)->(u32,u32,i32,i32);
    unsafe fn border_status(&self)->i32;
    unsafe fn resize(&self,sx:u32,sy:u32);
    unsafe fn window_observer(&self)->Weak<RefCell<window>>;
}
impl Pane for PaneRef {
    unsafe fn geometry(&self)->(u32,u32,i32,i32) {let state=&*self.get();(state.sx,state.sy,state.xoff,state.yoff)}
    unsafe fn border_status(&self)->i32 {let window=self.window_observer().upgrade().unwrap();let _loan=window.borrow();(*self.get()).border}
    unsafe fn resize(&self,sx:u32,sy:u32) {
        let window=self.window_observer().upgrade().unwrap();
        window.borrow_mut().resized.push((sx,sy));
        (*self.get()).sx=sx; (*self.get()).sy=sy;
        let callback=(*self.get()).on_resize.take();
        if let Some(callback)=callback {callback(&window);}
    }
    unsafe fn window_observer(&self)->Weak<RefCell<window>> {(*self.get()).window.clone()}
}
unsafe fn window_pane_scrollbar_reserve(pane:&window_pane)->i32 {
    let window=pane.window.upgrade().unwrap();let _loan=window.borrow();pane.reserve as i32
}
unsafe fn window_pane_get_pane_lines(pane:&window_pane)->u32 {
    let window=pane.window.upgrade().unwrap();let _loan=window.borrow();pane.lines
}
unsafe fn layout_resize_pane(pane:&PaneRef,_:layout_type,change:i32) {
    pane.window_observer().upgrade().unwrap().borrow_mut().resize_changes.push(change);
}
// SOURCE_OPERATIONS
// SOURCE_TESTS

unsafe fn leaf(pane:&PaneRef,sx:u32,sy:u32,xoff:i32,yoff:i32)->Box<layout_cell> {
    let mut cell=layout_create_cell();layout_make_leaf(&mut *cell,pane);
    layout_set_size(&mut *cell,sx,sy,xoff,yoff);cell
}
unsafe fn publish(window:&WindowRef,panes:&[PaneRef]) {
    for pane in panes {(*pane.get()).window=Rc::downgrade(window);}
    window.borrow_mut().panes=panes.iter().map(Rc::downgrade).collect();
}
#[test]
fn fix_panes_reads_the_new_tree_after_a_resize_callback_replaces_it() { unsafe {
    let window=window::new();let first=window_pane::new();let second=window_pane::new();
    publish(&window,&[first.clone(),second.clone()]);
    let mut root=layout_create_cell();layout_make_node(&mut *root,LAYOUT_LEFTRIGHT);
    layout_cells_push_back(&mut *root,leaf(&first,12,8,0,0));
    layout_cells_push_back(&mut *root,leaf(&second,8,8,13,0));window.borrow_mut().root=Some(root);
    let first_weak=Rc::downgrade(&first);let second_weak=Rc::downgrade(&second);
    (*first.get()).on_resize=Some(Box::new(move |window| {
        let mut root=layout_create_cell();layout_make_node(&mut *root,LAYOUT_LEFTRIGHT);
        layout_cells_push_back(&mut *root,leaf(&first_weak.upgrade().unwrap(),5,8,0,0));
        layout_cells_push_back(&mut *root,leaf(&second_weak.upgrade().unwrap(),15,8,6,0));
        window.borrow_mut().root=Some(root);
    }));
    layout_fix_panes(&window,None);
    assert_eq!(window.borrow().resized,[(12,8),(15,8)]);
    assert_eq!(second.geometry(),(15,8,6,0));
    assert_eq!(Rc::strong_count(&window),1);
}}
#[test]
fn fix_panes_applies_status_and_scrollbar_policy_before_resize() { unsafe {
    let window=window::new();let pane=window_pane::new();publish(&window,&[pane.clone()]);
    (*pane.get()).border=PANE_STATUS_TOP;(*pane.get()).reserve=true;
    (*pane.get()).scrollbar_style=ScrollbarStyle {width:3,pad:1};
    window.borrow_mut().root=Some(leaf(&pane,8,4,1,2));
    layout_fix_panes(&window,None);
    assert_eq!(pane.geometry(),(4,3,5,3));
    assert_ne!((*pane.get()).flags & PANE_REDRAWSCROLLBAR,0);
    assert_eq!(window.borrow().generation,1);
}}
#[test]
fn floating_clamp_and_fixed_resize_query_policy_without_a_tree_loan() { unsafe {
    let window=window::new();let pane=window_pane::new();publish(&window,&[pane.clone()]);
    let mut cell=leaf(&pane,30,22,25,17);cell.flags=LAYOUT_CELL_FLOATING;
    let id=cell.id();window.borrow_mut().root=Some(cell);
    layout_clamp_floating_panes(&window,20,12);
    assert_eq!((window.borrow().find(id).unwrap().g.sx,window.borrow().find(id).unwrap().g.sy),(20,12));
    (*pane.get()).lines=PANE_LINES_SINGLE;
    assert!(layout_resize_floating_pane_to(&pane,LAYOUT_LEFTRIGHT,18).is_ok());
    assert_eq!(window.borrow().find(id).unwrap().g.sx,16);
    assert_eq!(window.borrow().generation,1);
    assert!(layout_resize_floating_pane_to(&pane,LAYOUT_LEFTRIGHT,18).is_ok());
    assert_eq!(window.borrow().generation,1);
    assert!(layout_resize_floating_pane_to(&pane,LAYOUT_LEFTRIGHT,0).is_err());
    assert_eq!(window.borrow().find(id).unwrap().g.sx,16);
    assert_eq!(Rc::strong_count(&window),1);
}}
#[test]
fn assigning_a_saved_reservation_uses_its_original_window_after_reparenting() { unsafe {
    let original=window::new();let current=window::new();let pane=window_pane::new();
    (*pane.get()).window=Rc::downgrade(&current);
    let cell=layout_create_cell();let id=cell.id();original.borrow_mut().saved=Some(cell);
    layout_assign_pane(&original,id,&pane,1);
    assert!(original.borrow().find(id).unwrap().wp.ptr_eq(&Rc::downgrade(&pane)));
    assert_eq!((*pane.get()).layout_cell,Some(id));
    assert!((*pane.get()).window.ptr_eq(&Rc::downgrade(&current)));
    assert_eq!(Rc::strong_count(&original),1);assert_eq!(Rc::strong_count(&current),1);
}}
#[test]
fn resize_to_releases_cell_guard_before_the_resize_operation() { unsafe {
    let window=window::new();let first=window_pane::new();let second=window_pane::new();
    publish(&window,&[first.clone(),second.clone()]);
    let mut root=layout_create_cell();layout_make_node(&mut *root,LAYOUT_LEFTRIGHT);
    layout_cells_push_back(&mut *root,leaf(&first,10,8,0,0));
    layout_cells_push_back(&mut *root,leaf(&second,11,8,11,0));window.borrow_mut().root=Some(root);
    layout_resize_pane_to(&first,LAYOUT_LEFTRIGHT,15);
    layout_resize_pane_to(&second,LAYOUT_LEFTRIGHT,15);
    assert_eq!(window.borrow().resize_changes,[5,-4]);
    assert_eq!(Rc::strong_count(&window),1);
}}
'''

if __name__ == "__main__":
    unittest.main()
