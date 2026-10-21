//! Fresh bounded byte comparison for owned native bundles.
use std::io::{self, Read};
use std::path::Path;
const CHUNK: usize = 64 * 1024;

#[derive(Clone, Copy)]
pub(super) enum DestinationErrors {
    Reject,
    TreatAsMismatch,
}

fn read_retry(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    loop {
        match reader.read(buffer) {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            value => return value,
        }
    }
}
fn fill(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    let mut n = 0;
    while n < buffer.len() {
        let got = read_retry(reader, &mut buffer[n..])?;
        if got == 0 {
            break;
        }
        n += got;
    }
    Ok(n)
}
// Drain all bytes even after inequality, preserving detection of later read errors.
// A destination error may trigger capture, but source errors always propagate.
fn compare_readers<R: Read, S: Read>(
    source: &mut R,
    destination: io::Result<S>,
    policy: DestinationErrors,
) -> io::Result<bool> {
    let mut destination = destination;
    let mut a = [0; CHUNK];
    let mut b = [0; CHUNK];
    let mut equal = true;
    loop {
        let left = fill(source, &mut a)?;
        let right = match &mut destination {
            Ok(reader) => match fill(reader, &mut b) {
                Ok(n) => n,
                Err(e) => {
                    destination = Err(e);
                    0
                }
            },
            Err(_) => 0,
        };
        equal &= left == right && a[..left] == b[..right];
        if left == 0 && right == 0 {
            break;
        }
    }
    match destination {
        Ok(_) => Ok(equal),
        Err(_) if matches!(policy, DestinationErrors::TreatAsMismatch) => Ok(false),
        Err(e) => Err(e),
    }
}
pub(super) fn matches(
    source: &Path,
    destination: &Path,
    policy: DestinationErrors,
) -> io::Result<bool> {
    let mut source = std::fs::File::open(source)?;
    compare_readers(&mut source, std::fs::File::open(destination), policy)
}

#[cfg(test)]
#[path = "native_bytes_tests.rs"]
mod tests;
