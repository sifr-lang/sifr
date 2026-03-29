use std::fs;
use std::io;
use std::path::Path;

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
    let path = Path::new("/tmp/sifr_runtime_wave0_bytes_demo.bin");

    let (payload_ok, ints_ok) =
        match fs::write(path, b"runtime-wave0").and_then(|()| fs::read(path)) {
            Ok(loaded) => (
                loaded == b"runtime-wave0",
                ints_string(&loaded)
                    == "[114, 117, 110, 116, 105, 109, 101, 45, 119, 97, 118, 101, 48]",
            ),
            Err(_) => (false, false),
        };

    let cleanup_ok = cleanup(path).is_ok() && !path.exists();

    assert!(payload_ok);
    assert!(ints_ok);
    assert!(cleanup_ok);
    println!("ad_hoc_runtime_wave0_bytes_binary_io_contract_demo: ok");
}
