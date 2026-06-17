use std::fs;
use std::io;
use std::path::Path;

fn sum_bytes(data: &[u8]) -> i64 {
    data.iter().map(|&value| i64::from(value)).sum()
}

fn ints_string(data: &[u8]) -> String {
    format!(
        "[{}]",
        data.iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn contains_byte(data: &[u8], needle: i64) -> bool {
    u8::try_from(needle)
        .ok()
        .is_some_and(|value| data.contains(&value))
}

fn cleanup(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn main() {
    let payload = b"ffi-ready";

    let second_ok = payload.get(1).copied() == Some(102);
    let iter_ok = sum_bytes(payload) > 0;
    let contains_ok = contains_byte(payload, 102) && !contains_byte(payload, 512);
    let to_ints_ok = ints_string(payload) == "[102, 102, 105, 45, 114, 101, 97, 100, 121]";

    let path = Path::new("/tmp/sifr_bytes_readonly_bytes.bin");
    let io_ok = match fs::write(path, payload).and_then(|()| fs::read(path)) {
        Ok(loaded) => {
            loaded == payload
                && ints_string(&loaded) == "[102, 102, 105, 45, 114, 101, 97, 100, 121]"
        }
        Err(_) => false,
    };
    let cleanup_ok = cleanup(path).is_ok() && !path.exists();

    assert!(second_ok);
    assert!(iter_ok);
    assert!(contains_ok);
    assert!(to_ints_ok);
    assert!(io_ok);
    assert!(cleanup_ok);
}
