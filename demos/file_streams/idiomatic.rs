use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn cleanup(path: &Path) -> bool {
    !path.exists() || fs::remove_file(path).is_ok()
}

fn main() {
    let path = Path::new("/tmp/sifr_runtime_file_streams_demo.txt");

    let mut text_ok = false;
    let mut binary_ok = false;

    if fs::write(path, "alpha\nbeta").is_ok() {
        if let Ok(file) = fs::File::open(path) {
            let mut lines = BufReader::new(file).lines();
            let first = lines.next().and_then(Result::ok);
            let second = lines.next().and_then(Result::ok);
            let third = lines.next().and_then(Result::ok);
            text_ok = first.as_deref() == Some("alpha")
                && second.as_deref() == Some("beta")
                && third.is_none();
        }

        if fs::write(path, b"raw-bytes").is_ok() {
            binary_ok = fs::read(path).is_ok_and(|payload| payload == b"raw-bytes");
        }
    }

    let cleanup_ok = cleanup(path) && !path.exists();

    assert!(text_ok);
    assert!(binary_ok);
    assert!(cleanup_ok);
    println!("runtime_file_streams_hierarchy_demo: ok");
}
