use std::fs;
use std::path::Path;

fn ints_string(bytes: &[u8]) -> String {
    format!(
        "[{}]",
        bytes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn cleanup(path: &Path) -> bool {
    !path.exists() || fs::remove_file(path).is_ok()
}

fn main() {
    let path = Path::new("/tmp/sifr_bytes_bytes_file_io.bin");
    let payload = b"bytes_file_io";

    let (loaded_ok, ints_ok) = match fs::write(path, payload).and_then(|()| fs::read(path)) {
        Ok(loaded) => (
            loaded == payload,
            ints_string(&loaded) == "[98, 121, 116, 101, 115, 95, 102, 105, 108, 101, 95, 105, 111]",
        ),
        Err(_) => (false, false),
    };

    let cleanup_ok = cleanup(path) && !path.exists();

    assert!(loaded_ok);
    assert!(ints_ok);
    assert!(cleanup_ok);
}
