use super::{DestinationErrors, compare_readers};
use std::io::{self, Cursor, Read};

struct Fragmented {
    bytes: Cursor<Vec<u8>>,
    size: usize,
    calls: usize,
    fail_at: Option<u64>,
}
impl Fragmented {
    fn new(bytes: &[u8], size: usize) -> Self {
        Self {
            bytes: Cursor::new(bytes.to_vec()),
            size,
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
            return Err(io::Error::other("injected bundle read failure"));
        }
        let len = buffer.len().min(self.size);
        self.bytes.read(&mut buffer[..len])
    }
}

#[test]
fn comparisons_handle_boundaries_unequal_lengths_and_fragmented_reads() {
    for len in [0, 1, 65_535, 65_536, 65_537, 196_621] {
        let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
        assert!(
            compare_readers(
                &mut Fragmented::new(&data, 17),
                Ok(Fragmented::new(&data, 257)),
                DestinationErrors::Reject
            )
            .unwrap()
        );
        if len > 0 {
            let mut changed = data.clone();
            changed[len - 1] ^= 1;
            assert!(
                !compare_readers(
                    &mut Fragmented::new(&data, 257),
                    Ok(Fragmented::new(&changed, 17)),
                    DestinationErrors::Reject
                )
                .unwrap()
            );
        }
    }
    for (a, b) in [
        (b"".as_slice(), b"x".as_slice()),
        (b"x".as_slice(), b"".as_slice()),
        (b"abc".as_slice(), b"abcd".as_slice()),
    ] {
        assert!(
            !compare_readers(
                &mut Fragmented::new(a, 1),
                Ok(Fragmented::new(b, 2)),
                DestinationErrors::Reject
            )
            .unwrap()
        );
    }
}

#[test]
fn source_errors_after_mismatch_are_never_hidden_by_capture_policy() {
    let source = vec![7; 131_073];
    let different = vec![8; source.len()];
    for policy in [
        DestinationErrors::Reject,
        DestinationErrors::TreatAsMismatch,
    ] {
        let mut broken = Fragmented::new(&source, 1024);
        broken.fail_at = Some(70_000);
        assert_eq!(
            compare_readers(&mut broken, Ok(Fragmented::new(&different, 2048)), policy)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Other
        );
        let mut broken = Fragmented::new(&source, 1024);
        broken.fail_at = Some(70_000);
        assert_eq!(
            compare_readers(
                &mut broken,
                Err::<Cursor<Vec<u8>>, _>(io::Error::from(io::ErrorKind::PermissionDenied)),
                policy
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::Other
        );
    }
}

#[test]
fn destination_errors_reject_verification_but_require_capture() {
    let data = vec![7; 131_073];
    let different = vec![8; data.len()];
    for policy in [
        DestinationErrors::Reject,
        DestinationErrors::TreatAsMismatch,
    ] {
        let mut broken = Fragmented::new(&data, 1024);
        broken.fail_at = Some(70_000);
        let result = compare_readers(&mut Fragmented::new(&different, 2048), Ok(broken), policy);
        match policy {
            DestinationErrors::Reject => {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other)
            }
            DestinationErrors::TreatAsMismatch => assert!(!result.unwrap()),
        }
        let result = compare_readers(
            &mut Fragmented::new(&data, 1024),
            Err::<Cursor<Vec<u8>>, _>(io::Error::from(io::ErrorKind::PermissionDenied)),
            policy,
        );
        match policy {
            DestinationErrors::Reject => {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied)
            }
            DestinationErrors::TreatAsMismatch => assert!(!result.unwrap()),
        }
    }
}
