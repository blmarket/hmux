use refbox::Weak;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

// Keep this in its own integration-test binary so no other test allocates
// concurrently with the measured operations.
#[test]
fn empty_lifecycle_does_not_touch_allocator() {
    let before_alloc = ALLOCATIONS.load(Ordering::Relaxed);
    let before_dealloc = DEALLOCATIONS.load(Ordering::Relaxed);
    let empty = std::hint::black_box(Weak::<String>::new());
    let cloned = std::hint::black_box(empty.clone());
    let tried = std::hint::black_box(empty.try_clone().unwrap());
    let defaulted = std::hint::black_box(Weak::<String>::default());
    let raw = cloned.into_raw();
    drop(unsafe { Weak::from_raw(raw) });
    drop((empty, tried, defaulted));
    let after_alloc = ALLOCATIONS.load(Ordering::Relaxed);
    let after_dealloc = DEALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(before_alloc, after_alloc);
    assert_eq!(before_dealloc, after_dealloc);
}
