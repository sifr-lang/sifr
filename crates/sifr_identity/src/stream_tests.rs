use super::IdentityEncoder;
use std::io::{self, Cursor, Read};

struct Fragmented {
    bytes: Cursor<Vec<u8>>,
    calls: usize,
    fail_at: Option<u64>,
}
impl Fragmented {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Cursor::new(bytes.to_vec()),
            calls: 0,
            fail_at: None,
        }
    }
}
impl Read for Fragmented {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        if self.calls % 5 == 0 {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        if self.fail_at.is_some_and(|at| self.bytes.position() >= at) {
            return Err(io::Error::other("injected field read failure"));
        }
        let until_failure = self
            .fail_at
            .map_or(usize::MAX, |at| (at - self.bytes.position()) as usize);
        let len = buffer.len().min(19).min(until_failure);
        self.bytes.read(&mut buffer[..len])
    }
}

#[test]
fn streamed_fields_keep_slice_framing_through_short_reads_and_interruptions() {
    for len in [0, 1, 63, 64, 65_535, 65_536, 65_537, 196_621, 2_000_003] {
        let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
        let mut old = IdentityEncoder::new("stream-control");
        old.field("before", b"prefix");
        old.field("bytes", &data);
        old.field("after", b"suffix");
        let mut new = IdentityEncoder::new("stream-control");
        new.field("before", b"prefix");
        new.field_reader("bytes", len as u64, &mut Fragmented::new(&data))
            .unwrap();
        new.field("after", b"suffix");
        assert_eq!(old.finish(), new.finish(), "payload length {len}");
    }
}

#[test]
fn failed_fields_leave_the_encoder_unchanged_and_allow_a_valid_retry() {
    for (bytes, declared, fail_at, expected) in [
        (b"abc".as_slice(), 4, None, io::ErrorKind::UnexpectedEof),
        (b"abcd".as_slice(), 3, None, io::ErrorKind::InvalidData),
        (b"abc".as_slice(), 3, Some(1), io::ErrorKind::Other),
        (b"abc".as_slice(), 3, Some(3), io::ErrorKind::Other),
        (b"x".as_slice(), 0, None, io::ErrorKind::InvalidData),
    ] {
        let mut encoder = IdentityEncoder::new("failed-field");
        encoder.field("before", b"unchanged");
        let mut expected_encoder = encoder.clone();
        let mut reader = Fragmented::new(bytes);
        reader.fail_at = fail_at;
        assert_eq!(
            encoder
                .field_reader("failed", declared, &mut reader)
                .unwrap_err()
                .kind(),
            expected
        );
        assert_eq!(encoder.clone().finish(), expected_encoder.clone().finish());
        encoder
            .field_reader("valid", 3, &mut Cursor::new(b"abc"))
            .unwrap();
        expected_encoder.field("valid", b"abc");
        assert_eq!(encoder.finish(), expected_encoder.finish());
    }
}
