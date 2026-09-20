//! Raw-call-site stream handles. The registry owns storage and task lifetimes.
use super::{streams, BUFFERS};
use crate::src::shared::abi::size_t;
use crate::src::shared::event::{bufferevent, bufferevent_data_cb, bufferevent_event_cb};
use hmux_rt::{adapters::WatchInterest, ByteBuffer};
use std::ffi::{c_int, c_short, c_void};
use std::rc::Rc;
fn interest(flags: c_short) -> WatchInterest {
    match flags & 6 {
        2 => WatchInterest::Read,
        4 => WatchInterest::Write,
        _ => WatchInterest::ReadWrite,
    }
}
pub unsafe fn bufferevent_new(
    fd: c_int,
    read: bufferevent_data_cb,
    write: bufferevent_data_cb,
    error: bufferevent_event_cb,
    arg: *mut c_void,
) -> *mut bufferevent {
    let stream = Box::into_raw(Box::new(bufferevent {
        id: 0,
        input: std::ptr::null_mut(),
        output: std::ptr::null_mut(),
    }));
    let registry = streams();
    let read =
        read.map(|cb| Rc::new(move |_| unsafe { cb(stream, arg) }) as hmux_rt::stream::StreamCb);
    let write =
        write.map(|cb| Rc::new(move |_| unsafe { cb(stream, arg) }) as hmux_rt::stream::StreamCb);
    let error = error.map(|cb| {
        Rc::new(move |_, flags| unsafe { cb(stream, flags, arg) }) as hmux_rt::stream::StreamErrorCb
    });
    let id = registry.allocate(fd, read, write, error);
    (*stream).id = id;
    (*stream).input = registry.with_input(id, |b| b as *mut ByteBuffer).unwrap();
    (*stream).output = registry.with_output(id, |b| b as *mut ByteBuffer).unwrap();
    BUFFERS.with(|b| {
        let mut b = b.borrow_mut();
        b.insert((*stream).input as usize, id);
        b.insert((*stream).output as usize, id);
    });
    // The legacy constructor enables writes, but leaves reads disabled.
    registry.enable(id, WatchInterest::Write);
    stream
}
pub unsafe fn bufferevent_free(stream: *mut bufferevent) {
    if stream.is_null() {
        return;
    }
    let stream = Box::from_raw(stream);
    BUFFERS.with(|b| {
        let mut b = b.borrow_mut();
        b.remove(&(stream.input as usize));
        b.remove(&(stream.output as usize));
    });
    streams().free(stream.id);
}
pub unsafe fn stream_input(stream: *mut bufferevent) -> *mut ByteBuffer {
    (*stream).input
}
pub unsafe fn stream_output(stream: *mut bufferevent) -> *mut ByteBuffer {
    (*stream).output
}
pub unsafe fn bufferevent_get_output(stream: *mut bufferevent) -> *mut ByteBuffer {
    stream_output(stream)
}
pub unsafe fn bufferevent_enable(stream: *mut bufferevent, flags: c_short) -> c_int {
    if flags & 6 != 0 {
        streams().enable((*stream).id, interest(flags));
    }
    0
}
pub unsafe fn bufferevent_disable(stream: *mut bufferevent, flags: c_short) -> c_int {
    if flags & 6 != 0 {
        streams().disable((*stream).id, interest(flags));
    }
    0
}
pub unsafe fn bufferevent_write(
    stream: *mut bufferevent,
    data: *const c_void,
    size: size_t,
) -> c_int {
    let data = if size == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(data.cast::<u8>(), size)
    };
    if streams().write((*stream).id, data) {
        0
    } else {
        -1
    }
}
pub unsafe fn bufferevent_write_buffer(stream: *mut bufferevent, buffer: *mut ByteBuffer) -> c_int {
    if streams().write_buffer((*stream).id, &mut *buffer) {
        super::wake_buffer(buffer);
        0
    } else {
        -1
    }
}
pub unsafe fn bufferevent_setwatermark(
    stream: *mut bufferevent,
    flags: c_short,
    low: size_t,
    high: size_t,
) {
    let registry = streams();
    if flags & 2 != 0 {
        registry.set_read_watermark((*stream).id, low, high);
    }
    if flags & 4 != 0 {
        registry.set_watermark((*stream).id, low, high);
    }
}
