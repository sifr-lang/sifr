use std::fs;
use std::io;
use std::path::Path;

fn sum_bytes(data: &[u8]) -> i64 {
    data.iter().map(|&value| i64::from(value)).sum()
}

fn bytes_to_hex(data: &[u8]) -> String {
    data.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn contains_byte(data: &[u8], needle: i64) -> bool {
    u8::try_from(needle)
        .ok()
        .is_some_and(|value| data.contains(&value))
}

fn count_byte(data: &[u8], needle: i64) -> usize {
    u8::try_from(needle).ok().map_or(0, |value| {
        data.iter().filter(|&&item| item == value).count()
    })
}

fn bytes_from_hex(text: &str) -> Option<Vec<u8>> {
    let cleaned: String = text
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();
    if cleaned.len() % 2 != 0 || !cleaned.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let mut out = Vec::with_capacity(cleaned.len() / 2);
    let bytes = cleaned.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let chunk = std::str::from_utf8(&bytes[index..index + 2]).ok()?;
        out.push(u8::from_str_radix(chunk, 16).ok()?);
        index += 2;
    }
    Some(out)
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

fn cleanup(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn main() {
    let payload = b"binary_storage";
    let second_ok = payload.get(1).copied() == Some(105);
    let iter_ok = sum_bytes(payload) == 1497;
    let contains_ok = contains_byte(payload, 98) && !contains_byte(payload, 512);
    let count_ok = count_byte(payload, 98) == 1 && count_byte(payload, 512) == 0;
    let hex_ok = bytes_from_hex(&bytes_to_hex(payload)).as_deref() == Some(payload.as_slice());

    let path = Path::new("/tmp/sifr_bytes_binary_storage.bin");
    let io_ok = match fs::write(path, payload).and_then(|()| fs::read(path)) {
        Ok(loaded) => loaded == payload && ints_string(&loaded) == "[98, 105, 110, 97, 114, 121, 95, 115, 116, 111, 114, 97, 103, 101]",
        Err(_) => false,
    };
    let cleanup_ok = cleanup(path).is_ok() && !path.exists();

    assert!(second_ok);
    assert!(iter_ok);
    assert!(contains_ok);
    assert!(count_ok);
    assert!(hex_ok);
    assert!(io_ok);
    assert!(cleanup_ok);
}
