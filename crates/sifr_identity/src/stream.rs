//! Bounded, transactional encoding of a length-delimited field.
use super::IdentityEncoder;
use sha2::Digest;
use std::io::{self, Read};
const CHUNK: usize = 64 * 1024;

impl IdentityEncoder {
    /// Encode exactly `len` current bytes with the same framing as [`Self::field`].
    ///
    /// The reader must end at that boundary. Short/long input or a read error
    /// leaves this encoder unchanged; interruptions and short reads are retried.
    pub fn field_reader(&mut self, name: &str, len: u64, reader: &mut impl Read) -> io::Result<()> {
        let mut next = self.clone();
        next.0.update((name.len() as u64).to_le_bytes());
        next.0.update(name.as_bytes());
        next.0.update(len.to_le_bytes());
        let mut remaining = len;
        let mut buffer = [0; CHUNK];
        while remaining > 0 {
            let requested = remaining.min(CHUNK as u64) as usize;
            let n = read_retry(reader, &mut buffer[..requested])?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "short identity field",
                ));
            }
            next.0.update(&buffer[..n]);
            remaining -= n as u64;
        }
        if read_retry(reader, &mut buffer[..1])? != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "long identity field",
            ));
        }
        *self = next;
        Ok(())
    }
}

fn read_retry(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    loop {
        match reader.read(buffer) {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            value => return value,
        }
    }
}
