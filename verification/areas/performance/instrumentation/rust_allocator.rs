// Successful Rust GlobalAlloc requests while generated main executes.
// Loader/libc allocations and process cleanup after main are outside this scope.
use std::alloc::{GlobalAlloc, Layout, System};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

struct ObservedSystem;
#[global_allocator]
static ALLOCATOR: ObservedSystem = ObservedSystem;
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static REALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for ObservedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc(layout) };
        if !result.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::SeqCst);
        }
        result
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc_zeroed(layout) };
        if !result.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::SeqCst);
        }
        result
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(pointer, layout, size) };
        if !result.is_null() {
            REALLOCATIONS.fetch_add(1, Ordering::SeqCst);
            REALLOCATED_BYTES.fetch_add(size as u64, Ordering::SeqCst);
        }
        result
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        DEALLOCATIONS.fetch_add(1, Ordering::SeqCst);
    }
}

fn snapshot() -> [u64; 5] {
    [
        ALLOCATIONS.load(Ordering::SeqCst),
        ALLOCATED_BYTES.load(Ordering::SeqCst),
        REALLOCATIONS.load(Ordering::SeqCst),
        REALLOCATED_BYTES.load(Ordering::SeqCst),
        DEALLOCATIONS.load(Ordering::SeqCst),
    ]
}

pub struct MainObservation([u64; 5]);
impl MainObservation {
    pub fn begin() -> Self {
        Self(snapshot())
    }
}
impl Drop for MainObservation {
    fn drop(&mut self) {
        // Snapshot before reporting: environment lookup, formatting and file I/O
        // may allocate and are intentionally excluded from the measured delta.
        let after = snapshot();
        let mut delta = [0; 5];
        for (index, value) in after.iter().enumerate() {
            let Some(difference) = value.checked_sub(self.0[index]) else {
                std::process::exit(2);
            };
            delta[index] = difference;
        }
        let Ok(path) = std::env::var("SIFR_ALLOCATION_RECEIPT") else {
            std::process::exit(2);
        };
        let payload = format!(
            "{{\"schema_version\":1,\"allocation_calls\":{},\"allocation_requested_bytes\":{},\"reallocation_calls\":{},\"reallocation_requested_bytes\":{},\"deallocation_calls\":{}}}\n",
            delta[0], delta[1], delta[2], delta[3], delta[4]
        );
        let result = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .and_then(|mut file| {
                file.write_all(payload.as_bytes())
                    .and_then(|()| file.sync_all())
            });
        if result.is_err() {
            std::process::exit(2);
        }
    }
}
