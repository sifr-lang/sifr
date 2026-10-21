//! Bounded logical read/seek view over immutable physical frames.
use std::io::{self, Read, Seek, SeekFrom};
use std::ops::Range;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
struct Frame {
    compressed: Range<usize>,
    logical: Range<usize>,
    decoded: Option<Vec<u8>>,
}
pub(super) struct PhysicalInput {
    prefix: Vec<u8>,
    compressed: Vec<u8>,
    frames: Vec<Frame>,
    expanded: usize,
    position: u64,
    pub(super) payload_decode_us: Arc<AtomicU64>,
}
impl PhysicalInput {
    pub(super) fn new(
        prefix: Vec<u8>,
        compressed: Vec<u8>,
        ranges: Vec<Range<usize>>,
        lengths: [usize; 2],
        expanded: usize,
    ) -> Self {
        let mut next = prefix.len();
        let frames = ranges
            .into_iter()
            .zip(lengths)
            .map(|(compressed, length)| {
                let start = next;
                next += length;
                Frame {
                    compressed,
                    logical: start..next,
                    decoded: None,
                }
            })
            .collect();
        Self {
            prefix,
            compressed,
            frames,
            expanded,
            position: 0,
            payload_decode_us: Arc::new(AtomicU64::new(0)),
        }
    }
    fn payload(&mut self, position: usize) -> io::Result<&[u8]> {
        let index = self
            .frames
            .iter()
            .position(|frame| frame.logical.contains(&position))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "unmapped payload position")
            })?;
        let frame = &mut self.frames[index];
        if frame.decoded.is_none() {
            let started = std::time::Instant::now();
            let bound = frame.logical.len();
            let payload =
                zstd::bulk::decompress(&self.compressed[frame.compressed.clone()], bound)?;
            if payload.len() != bound {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "expanded payload length differs from bounded layout",
                ));
            }
            frame.decoded = Some(payload);
            let elapsed = u64::try_from(started.elapsed().as_micros())
                .unwrap_or(u64::MAX)
                .max(1);
            let _ = self.payload_decode_us.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |previous| Some(previous.saturating_add(elapsed)),
            );
        }
        if self
            .frames
            .iter()
            .all(|frame| frame.decoded.is_some() || frame.logical.is_empty())
        {
            self.compressed = Vec::new();
        }
        let frame = &self.frames[index];
        frame
            .decoded
            .as_deref()
            .map(|bytes| &bytes[position - frame.logical.start..])
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
            self.payload(position)?
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
