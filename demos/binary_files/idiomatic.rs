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
    let path = Path::new("/tmp/sifr_runtime_binary_files_demo.bin");

    let (payload_ok, ints_ok) =
        match fs::write(path, b"runtime-binary_files").and_then(|()| fs::read(path)) {
            Ok(loaded) => (
                loaded == b"runtime-binary_files",
                ints_string(&loaded)
                    == "[114, 117, 110, 116, 105, 109, 101, 45, 98, 105, 110, 97, 114, 121, 95, 102, 105, 108, 101, 115]",
            ),
            Err(_) => (false, false),
        };

    let cleanup_ok = cleanup(path).is_ok() && !path.exists();

    assert!(payload_ok);
    assert!(ints_ok);
    assert!(cleanup_ok);
    println!("runtime_binary_files_binary_io_demo: ok");
}
