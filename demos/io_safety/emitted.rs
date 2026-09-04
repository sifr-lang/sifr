// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::IOError;
    pub(super) fn read_text(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(super) fn read_lines(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::read_lines(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn append_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::append_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(super) fn getcwd() -> Result<String, IOError> {
        ::sifr_stdlib::fs::getcwd().map_err(sifr_generated_io_err)
    }
    pub(super) fn listdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn mkdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn copy_file(src: &str, dst: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::copy_file(src, dst).map_err(sifr_generated_io_err)
    }
    pub(super) fn rmdir_all(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir_all(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn copy(src: &str, dst: &str) -> Result<(), IOError> {
        copy_file(src, dst)
    }
    pub(super) fn rmtree(path: &str) -> Result<(), IOError> {
        rmdir_all(path)
    }
    pub(super) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let sifr_generated_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match sifr_generated_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => "PermissionDenied".to_string(),
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => "DirectoryNotEmpty".to_string(),
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
}
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
}
use crate::sifr_generated_generated_support::{
    append_text, copy, getcwd, listdir, mkdir, read_lines, read_text, remove_file, rmtree,
    write_text,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
fn demo_safe_read_write() {
    println!("=== Safe File Read/Write ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        write_text("/tmp/sifr_io_demo.txt", "hello from sifr")?;
        let content: String = read_text("/tmp/sifr_io_demo.txt")?;
        println!("read: {content}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
}
fn demo_file_not_found() {
    println!("=== File Not Found (no panic) ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _ = read_text("/tmp/sifr_io_demo_missing_file.txt")?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught IOError: {}", e.message);
    }
}
fn demo_directory_ops() {
    println!("=== Directory Operations ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        mkdir("/tmp/sifr_io_demo_dir")?;
        write_text("/tmp/sifr_io_demo_dir/test.txt", "inside dir")?;
        let entries: Vec<String> = listdir("/tmp/sifr_io_demo_dir")?;
        println!("entries: {}", SifrInt::from(entries.len()));
        let content: String = read_text("/tmp/sifr_io_demo_dir/test.txt")?;
        println!("file in dir: {content}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _ = listdir("/tmp/sifr_io_demo_nonexistent_xyz")?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught listdir IOError: {}", e.message);
    }
}
fn demo_copy_and_cleanup() {
    println!("=== Copy and Cleanup ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        copy("/tmp/sifr_io_demo.txt", "/tmp/sifr_io_demo_copy.txt")?;
        let copy_content: String = read_text("/tmp/sifr_io_demo_copy.txt")?;
        println!("copy: {copy_content}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        remove_file("/tmp/sifr_io_demo.txt")?;
        remove_file("/tmp/sifr_io_demo_copy.txt")?;
        rmtree("/tmp/sifr_io_demo_dir")?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("cleanup error: {}", e.message);
    }
}
fn demo_read_lines() {
    println!("=== Read Lines ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        write_text("/tmp/sifr_io_demo_lines.txt", "line1\nline2\nline3")?;
        let lines: Vec<String> = read_lines("/tmp/sifr_io_demo_lines.txt")?;
        println!("line count: {}", SifrInt::from(lines.len()));
        remove_file("/tmp/sifr_io_demo_lines.txt")?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
}
fn demo_append() {
    println!("=== Append Text ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        write_text("/tmp/sifr_io_demo_append.txt", "first")?;
        append_text("/tmp/sifr_io_demo_append.txt", " second")?;
        let content: String = read_text("/tmp/sifr_io_demo_append.txt")?;
        println!("appended: {content}");
        remove_file("/tmp/sifr_io_demo_append.txt")?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
}
fn demo_getcwd() {
    println!("=== Get Current Directory ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _ = getcwd()?;
        println!("getcwd succeeded");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("getcwd error: {}", e.message);
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
