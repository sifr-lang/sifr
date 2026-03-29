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
    let path = Path::new("/tmp/sifr_ad_hoc_bytes_wave3_demo.bin");
    let payload = b"wave3";

    let (loaded_ok, ints_ok) = match fs::write(path, payload).and_then(|()| fs::read(path)) {
        Ok(loaded) => (
            loaded == payload,
            ints_string(&loaded) == "[119, 97, 118, 101, 51]",
        ),
        Err(_) => (false, false),
    };

    let cleanup_ok = cleanup(path) && !path.exists();

    assert!(loaded_ok);
    assert!(ints_ok);
    assert!(cleanup_ok);
}
