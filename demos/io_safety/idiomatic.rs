use std::fs;
use std::io::Write;

fn write_text(path: &str, text: &str) -> std::io::Result<()> {
    fs::write(path, text)
}

fn read_text(path: &str) -> std::io::Result<String> {
    fs::read_to_string(path)
}

fn listdir(path: &str) -> std::io::Result<Vec<String>> {
    fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().to_string()))
        .collect()
}

fn copy(src: &str, dst: &str) -> std::io::Result<()> {
    fs::copy(src, dst).map(|_| ())
}

fn rmtree(path: &str) -> std::io::Result<()> {
    fs::remove_dir_all(path)
}

fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    Ok(read_text(path)?
        .lines()
        .map(|line| line.to_string())
        .collect())
}

fn append_text(path: &str, text: &str) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    file.write_all(text.as_bytes())
}

fn demo_safe_read_write() {
    println!("=== Safe File Read/Write ===");
    match (|| -> std::io::Result<()> {
        write_text("/tmp/sifr_io_demo.txt", "hello from sifr")?;
        let content = read_text("/tmp/sifr_io_demo.txt")?;
        println!("read: {}", content);
        Ok(())
    })() {
        Ok(()) => {}
        Err(error) => println!("error: {}", error),
    }
}

fn demo_file_not_found() {
    println!("=== File Not Found (no panic) ===");
    match read_text("/tmp/sifr_io_demo_missing_file.txt") {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("caught IOError: {}", error),
    }
}

fn demo_directory_ops() {
    println!("=== Directory Operations ===");
    match (|| -> std::io::Result<()> {
        fs::create_dir_all("/tmp/sifr_io_demo_dir")?;
        write_text("/tmp/sifr_io_demo_dir/test.txt", "inside dir")?;
        let entries = listdir("/tmp/sifr_io_demo_dir")?;
        println!("entries: {}", entries.len());
        let content = read_text("/tmp/sifr_io_demo_dir/test.txt")?;
        println!("file in dir: {}", content);
        Ok(())
    })() {
        Ok(()) => {}
        Err(error) => println!("error: {}", error),
    }

    match listdir("/tmp/sifr_io_demo_nonexistent_xyz") {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("caught listdir IOError: {}", error),
    }
}

fn demo_copy_and_cleanup() {
    println!("=== Copy and Cleanup ===");
    match (|| -> std::io::Result<()> {
        copy("/tmp/sifr_io_demo.txt", "/tmp/sifr_io_demo_copy.txt")?;
        let content = read_text("/tmp/sifr_io_demo_copy.txt")?;
        println!("copy: {}", content);
        Ok(())
    })() {
        Ok(()) => {}
        Err(error) => println!("error: {}", error),
    }

    match (|| -> std::io::Result<()> {
        fs::remove_file("/tmp/sifr_io_demo.txt")?;
        fs::remove_file("/tmp/sifr_io_demo_copy.txt")?;
        rmtree("/tmp/sifr_io_demo_dir")?;
        Ok(())
    })() {
        Ok(()) => {}
        Err(error) => println!("cleanup error: {}", error),
    }
}

fn demo_read_lines() {
    println!("=== Read Lines ===");
    match (|| -> std::io::Result<()> {
        write_text("/tmp/sifr_io_demo_lines.txt", "line1\nline2\nline3")?;
        let lines = read_lines("/tmp/sifr_io_demo_lines.txt")?;
        println!("line count: {}", lines.len());
        fs::remove_file("/tmp/sifr_io_demo_lines.txt")?;
        Ok(())
    })() {
        Ok(()) => {}
        Err(error) => println!("error: {}", error),
    }
}

fn demo_append() {
    println!("=== Append Text ===");
    match (|| -> std::io::Result<()> {
        write_text("/tmp/sifr_io_demo_append.txt", "first")?;
        append_text("/tmp/sifr_io_demo_append.txt", " second")?;
        let content = read_text("/tmp/sifr_io_demo_append.txt")?;
        println!("appended: {}", content);
        fs::remove_file("/tmp/sifr_io_demo_append.txt")?;
        Ok(())
    })() {
        Ok(()) => {}
        Err(error) => println!("error: {}", error),
    }
}

fn demo_getcwd() {
    println!("=== Get Current Directory ===");
    match std::env::current_dir() {
        Ok(_) => println!("getcwd succeeded"),
        Err(error) => println!("getcwd error: {}", error),
    }
}

fn main() {
    demo_safe_read_write();
    demo_file_not_found();
    demo_directory_ops();
    demo_copy_and_cleanup();
    demo_read_lines();
    demo_append();
    demo_getcwd();
    println!("=== All I/O safety demos passed ===");
}
