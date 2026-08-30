use crate::types::{grid_cell_entry, grid_cell_entry_union, grid_extd_entry, time_t, u_int};
use ::core::ffi::c_int;
use ::core::ptr::NonNull;
use ::core::slice;
use ::std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};

/// An entry with nothing in it, the way a freshly taken cell reads.
const EMPTY_ENTRY: grid_cell_entry = grid_cell_entry {
    c2rust_unnamed: grid_cell_entry_union { offset: 0 },
    flags: 0,
};

/// One line of a grid: its cells, its extended cells, and how far into them
/// anything was written.
///
/// The two arrays are owned here rather than held in `Vec`s so that a line
/// costs what tmux's `struct grid_line` costs. A `Vec` spends a pointer-sized
/// capacity and a pointer-sized length on each array where tmux spends one
/// `u_int`, and a grid carries one of these for every line of its history.
/// `cellsize` and `extdsize` are therefore the allocation as well as the
/// length, exactly as they are in the C, which is also what makes
/// `#{history_bytes}` able to price a line as `size_of::<grid_line>()`.
pub struct grid_line {
    celldata: NonNull<grid_cell_entry>,
    extddata: NonNull<grid_extd_entry>,
    cellsize: u_int,
    extdsize: u_int,
    pub cellused: u_int,
    pub flags: c_int,
    pub time: time_t,
}

/// The line holds what tmux's does, in the order that leaves no padding: the
/// same two pointers, three `u_int`s, `int` and `time_t`.
const _: () = assert!(
    ::core::mem::size_of::<grid_line>()
        == 2 * ::core::mem::size_of::<*const u8>()
            + 3 * ::core::mem::size_of::<u_int>()
            + ::core::mem::size_of::<c_int>()
            + ::core::mem::size_of::<time_t>()
);

/// Take `to` items over the `from` the pointer already holds. A count of none
/// holds no allocation at all, the way a null one did in the C.
unsafe fn resize<T>(ptr: NonNull<T>, from: u_int, to: u_int) -> NonNull<T> {
    if to == from {
        return ptr;
    }
    let old = Layout::array::<T>(from as usize).unwrap();
    if to == 0 {
        unsafe { dealloc(ptr.as_ptr().cast(), old) };
        return NonNull::dangling();
    }
    let new = Layout::array::<T>(to as usize).unwrap();
    let got = if from == 0 {
        unsafe { alloc(new) }
    } else {
        unsafe { realloc(ptr.as_ptr().cast(), old, new.size()) }
    };
    match NonNull::new(got.cast::<T>()) {
        Some(got) => got,
        None => handle_alloc_error(new),
    }
}

impl grid_line {
    /// A line with nothing in it.
    pub fn new() -> grid_line {
        grid_line {
            celldata: NonNull::dangling(),
            extddata: NonNull::dangling(),
            cellsize: 0,
            extdsize: 0,
            cellused: 0,
            flags: 0,
            time: 0,
        }
    }

    /// How many cells the line has room for, tmux's `cellsize`.
    pub fn cellsize(&self) -> u_int {
        self.cellsize
    }

    /// How many extended cells the line holds, tmux's `extdsize`.
    pub fn extdsize(&self) -> u_int {
        self.extdsize
    }

    pub fn celldata(&self) -> &[grid_cell_entry] {
        unsafe { slice::from_raw_parts(self.celldata.as_ptr(), self.cellsize as usize) }
    }

    pub fn celldata_mut(&mut self) -> &mut [grid_cell_entry] {
        unsafe { slice::from_raw_parts_mut(self.celldata.as_ptr(), self.cellsize as usize) }
    }

    pub fn extddata(&self) -> &[grid_extd_entry] {
        unsafe { slice::from_raw_parts(self.extddata.as_ptr(), self.extdsize as usize) }
    }

    pub fn extddata_mut(&mut self) -> &mut [grid_extd_entry] {
        unsafe { slice::from_raw_parts_mut(self.extddata.as_ptr(), self.extdsize as usize) }
    }

    /// The cells and the extended cells together. They are separate
    /// allocations, so a walk over the entries can read and rewrite the
    /// extended cells they point at.
    pub fn parts_mut(&mut self) -> (&mut [grid_cell_entry], &mut [grid_extd_entry]) {
        unsafe {
            (
                slice::from_raw_parts_mut(self.celldata.as_ptr(), self.cellsize as usize),
                slice::from_raw_parts_mut(self.extddata.as_ptr(), self.extdsize as usize),
            )
        }
    }

    /// Give the line room for `sx` cells. Any it gains read as empty; any it
    /// loses are given back, which the C left to the next line-wide free.
    pub fn resize_cells(&mut self, sx: u_int) {
        let from = self.cellsize;
        self.celldata = unsafe { resize(self.celldata, from, sx) };
        self.cellsize = sx;
        if sx > from {
            self.celldata_mut()[from as usize..].fill(EMPTY_ENTRY);
        }
    }

    /// Take one more extended cell, at the end, and answer where it went.
    pub fn push_extended(&mut self) -> u_int {
        let at = self.extdsize;
        self.extddata = unsafe { resize(self.extddata, at, at + 1) };
        self.extdsize = at + 1;
        self.extddata_mut()[at as usize] = grid_extd_entry {
            data: 0,
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        };
        at
    }

    /// Keep exactly these extended cells and give back the rest.
    pub fn set_extended(&mut self, entries: &[grid_extd_entry]) {
        let to = entries.len() as u_int;
        self.extddata = unsafe { resize(self.extddata, self.extdsize, to) };
        self.extdsize = to;
        self.extddata_mut().copy_from_slice(entries);
    }
}

impl Clone for grid_line {
    fn clone(&self) -> grid_line {
        let mut gl = grid_line::new();
        gl.resize_cells(self.cellsize);
        gl.celldata_mut().copy_from_slice(self.celldata());
        gl.set_extended(self.extddata());
        gl.cellused = self.cellused;
        gl.flags = self.flags;
        gl.time = self.time;
        gl
    }
}

impl Drop for grid_line {
    fn drop(&mut self) {
        unsafe {
            resize(self.celldata, self.cellsize, 0);
            resize(self.extddata, self.extdsize, 0);
        }
    }
}
