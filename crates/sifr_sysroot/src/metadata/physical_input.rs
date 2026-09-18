//! Bounded logical read/seek view over the two immutable physical frames.
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub(super) struct PhysicalInput {
    prefix: Vec<u8>,
    compressed: Vec<u8>,
    payload_start: usize,
    expanded: usize,
    payload: Option<Vec<u8>>,
    position: u64,
    pub(super) payload_decode_us: Arc<AtomicU64>,
}
impl PhysicalInput {
    pub(super) fn new(
        prefix: Vec<u8>,
        compressed: Vec<u8>,
        payload_start: usize,
        expanded: usize,
    ) -> Self {
        Self {
            prefix,
            compressed,
            payload_start,
            expanded,
            payload: None,
            position: 0,
            payload_decode_us: Arc::new(AtomicU64::new(0)),
        }
    }
    fn payload(&mut self) -> io::Result<&[u8]> {
        if self.payload.is_none() {
            let started = std::time::Instant::now();
            let bound = self.expanded - self.prefix.len();
            let payload = zstd::bulk::decompress(&self.compressed[self.payload_start..], bound)?;
            if payload.len() != bound {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "expanded payload length differs from bounded layout",
                ));
            }
            self.payload = Some(payload);
            self.compressed = Vec::new();
            self.payload_decode_us.store(
                u64::try_from(started.elapsed().as_micros())
                    .unwrap_or(u64::MAX)
                    .max(1),
                Ordering::Relaxed,
            );
        }
        self.payload
            .as_deref()
            .ok_or_else(|| io::Error::other("payload decompression did not produce bytes"))
    }
}
impl Read for PhysicalInput {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() || self.position >= self.expanded as u64 {
            return Ok(0);
        }
        let position = usize::try_from(self.position).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "read position exceeds address space",
            )
        })?;
        let bytes = if position < self.prefix.len() {
            &self.prefix[position..]
        } else {
            let offset = position - self.prefix.len();
            &self.payload()?[offset..]
        };
        let count = out.len().min(bytes.len());
        out[..count].copy_from_slice(&bytes[..count]);
        self.position += count as u64;
        Ok(count)
    }
}
impl Seek for PhysicalInput {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let position = match position {
            SeekFrom::Start(position) => i128::from(position),
            SeekFrom::End(offset) => self.expanded as i128 + i128::from(offset),
            SeekFrom::Current(offset) => i128::from(self.position) + i128::from(offset),
        };
        self.position = u64::try_from(position).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek outside addressable logical input",
            )
        })?;
        Ok(self.position)
    }
}
