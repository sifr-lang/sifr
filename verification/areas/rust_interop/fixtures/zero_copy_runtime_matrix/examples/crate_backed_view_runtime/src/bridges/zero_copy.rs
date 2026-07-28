use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::Bytes;
use memmap2::Mmap;
use sifr_runtime::interop::{Handle, HandleStateError};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

static ACTIVE_VIEWS: AtomicUsize = AtomicUsize::new(0);
static RELEASED_VIEWS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct ViewError {
    message: String,
}

impl ViewError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn context(context: &str, error: impl fmt::Display) -> Self {
        Self::new(format!("{context}: {error}"))
    }
}

impl fmt::Display for ViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ViewError {}

#[repr(C)]
#[derive(Debug, FromBytes, Immutable, IntoBytes, KnownLayout)]
struct Packet {
    tag: u32,
    length: u32,
}

#[derive(Debug)]
pub struct CrateBackedView {
    bytes_view: Bytes,
    bytes_pointer: usize,
    mmap_view: Mmap,
    mmap_pointer: usize,
    words: [u32; 2],
    packet: Packet,
}

impl Drop for CrateBackedView {
    fn drop(&mut self) {
        ACTIVE_VIEWS.fetch_sub(1, Ordering::SeqCst);
        RELEASED_VIEWS.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn create(mut data: Vec<u8>) -> Result<Handle<CrateBackedView>, ViewError> {
    if data.is_empty() {
        return Err(ViewError::new("zero-copy owner must not be empty"));
    }
    if ACTIVE_VIEWS.load(Ordering::SeqCst) != 0 {
        return Err(ViewError::new(
            "a previous zero-copy view remained active before creation",
        ));
    }
    RELEASED_VIEWS.store(0, Ordering::SeqCst);

    data[0] = b'S';
    let input_pointer = data.as_ptr() as usize;
    let owner = Bytes::from(data);
    if owner.as_ptr() as usize != input_pointer {
        return Err(ViewError::new(
            "bytes::Bytes did not retain the moved Vec allocation",
        ));
    }
    let bytes_view = owner.slice(..);
    let bytes_pointer = bytes_view.as_ptr() as usize;
    drop(owner);

    let mut mutable_map =
        memmap2::MmapMut::map_anon(8).map_err(|error| ViewError::context("anonymous mmap", error))?;
    mutable_map.copy_from_slice(b"mmapview");
    let mutable_pointer = mutable_map.as_ptr() as usize;
    let mmap_view = mutable_map
        .make_read_only()
        .map_err(|error| ViewError::context("read-only mmap transition", error))?;
    let mmap_pointer = mmap_view.as_ptr() as usize;
    if mutable_pointer != mmap_pointer {
        return Err(ViewError::new(
            "memmap2 changed allocation while sealing the mutable owner",
        ));
    }

    ACTIVE_VIEWS.fetch_add(1, Ordering::SeqCst);
    Ok(Handle::new(CrateBackedView {
        bytes_view,
        bytes_pointer,
        mmap_view,
        mmap_pointer,
        words: [0x7266_6953, 0x7765_6976],
        packet: Packet { tag: 7, length: 8 },
    }))
}

pub fn observe(view: &Handle<CrateBackedView>) -> Result<String, ViewError> {
    let view = view.inner_ref().map_err(handle_error)?;
    if view.bytes_view.as_ptr() as usize != view.bytes_pointer
        || view.bytes_view.first().copied() != Some(b'S')
    {
        return Err(ViewError::new(
            "bytes view lost its alias or retained owner lifetime",
        ));
    }
    if view.mmap_view.as_ptr() as usize != view.mmap_pointer
        || view.mmap_view.as_ref() != b"mmapview"
    {
        return Err(ViewError::new(
            "read-only memmap view lost its alias or owner",
        ));
    }

    let word_bytes: &[u8] = bytemuck::cast_slice(&view.words);
    if word_bytes.as_ptr() as usize != view.words.as_ptr() as usize {
        return Err(ViewError::new("bytemuck cast introduced a copy"));
    }
    let packet_bytes = view.packet.as_bytes();
    if packet_bytes.as_ptr() as usize != std::ptr::from_ref(&view.packet) as usize {
        return Err(ViewError::new("zerocopy view introduced a copy"));
    }

    Ok("bytes=alias+owner;memmap2=alias+readonly;bytemuck=alias;zerocopy=alias;mutation=exclusive;send-sync=required".to_string())
}

pub fn close(mut view: Handle<CrateBackedView>) -> Result<(), ViewError> {
    view.inner_ref().map_err(handle_error)?;
    view.mark_closed(sifr_runtime::interop::__generated_glue::token());
    Ok(())
}

pub fn release_observation() -> Result<String, ViewError> {
    let active = ACTIVE_VIEWS.load(Ordering::SeqCst);
    let released = RELEASED_VIEWS.load(Ordering::SeqCst);
    if active != 0 || released != 1 {
        return Err(ViewError::new(format!(
            "unexpected release state: released={released}, active={active}"
        )));
    }
    Ok(format!("released={released};active={active}"))
}

fn handle_error(error: HandleStateError) -> ViewError {
    ViewError::context("zero-copy view state", error)
}
