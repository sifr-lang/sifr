// src/main.rs
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
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
fn read_text(path: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::read_text(path).map_err(sifr_generated_io_err)
}
fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
}
fn read_lines(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::read_lines(path).map_err(sifr_generated_io_err)
}
fn append_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::append_text(path, content).map_err(sifr_generated_io_err)
}
fn getcwd() -> Result<String, IOError> {
    ::sifr_stdlib::fs::getcwd().map_err(sifr_generated_io_err)
}
fn listdir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path).map_err(sifr_generated_io_err)
}
fn mkdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path).map_err(sifr_generated_io_err)
}
fn remove_file(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
}
fn copy_file(src: &str, dst: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::copy_file(src, dst).map_err(sifr_generated_io_err)
}
fn rmdir_all(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir_all(path).map_err(sifr_generated_io_err)
}
fn copy(src: &str, dst: &str) -> Result<(), IOError> {
    copy_file(src, dst)
}
fn rmtree(path: &str) -> Result<(), IOError> {
    rmdir_all(path)
}
fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
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
fn demo_safe_read_write() {
    println!("=== Safe File Read/Write ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        write_text(
            &"/tmp/sifr_io_demo.txt".to_string(),
            &"hello from sifr".to_string(),
        )?;
        let content: String = read_text(&"/tmp/sifr_io_demo.txt".to_string())?;
        println!("read: {content}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
}
fn demo_file_not_found() {
    println!("=== File Not Found (no panic) ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _data: String = read_text(&"/tmp/sifr_io_demo_missing_file.txt".to_string())?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("caught IOError: {}", e.message.clone());
    }
}
fn demo_directory_ops() {
    println!("=== Directory Operations ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        mkdir(&"/tmp/sifr_io_demo_dir".to_string())?;
        write_text(
            &"/tmp/sifr_io_demo_dir/test.txt".to_string(),
            &"inside dir".to_string(),
        )?;
        let entries: Vec<String> = listdir(&"/tmp/sifr_io_demo_dir".to_string())?;
        println!("entries: {}", SifrInt::from(entries.len()));
        let content: String = read_text(&"/tmp/sifr_io_demo_dir/test.txt".to_string())?;
        println!("file in dir: {content}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _bad_entries: Vec<String> = listdir(&"/tmp/sifr_io_demo_nonexistent_xyz".to_string())?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("caught listdir IOError: {}", e.message.clone());
    }
}
fn demo_copy_and_cleanup() {
    println!("=== Copy and Cleanup ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        copy(
            &"/tmp/sifr_io_demo.txt".to_string(),
            &"/tmp/sifr_io_demo_copy.txt".to_string(),
        )?;
        let copy_content: String = read_text(&"/tmp/sifr_io_demo_copy.txt".to_string())?;
        println!("copy: {copy_content}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        remove_file(&"/tmp/sifr_io_demo.txt".to_string())?;
        remove_file(&"/tmp/sifr_io_demo_copy.txt".to_string())?;
        rmtree(&"/tmp/sifr_io_demo_dir".to_string())?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("cleanup error: {}", e.message.clone());
    }
}
fn demo_read_lines() {
    println!("=== Read Lines ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        write_text(
            &"/tmp/sifr_io_demo_lines.txt".to_string(),
            &"line1\nline2\nline3".to_string(),
        )?;
        let lines: Vec<String> = read_lines(&"/tmp/sifr_io_demo_lines.txt".to_string())?;
        println!("line count: {}", SifrInt::from(lines.len()));
        remove_file(&"/tmp/sifr_io_demo_lines.txt".to_string())?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
}
fn demo_append() {
    println!("=== Append Text ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        write_text(
            &"/tmp/sifr_io_demo_append.txt".to_string(),
            &"first".to_string(),
        )?;
        append_text(
            &"/tmp/sifr_io_demo_append.txt".to_string(),
            &" second".to_string(),
        )?;
        let content: String = read_text(&"/tmp/sifr_io_demo_append.txt".to_string())?;
        println!("appended: {content}");
        remove_file(&"/tmp/sifr_io_demo_append.txt".to_string())?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
}
fn demo_getcwd() {
    println!("=== Get Current Directory ===");
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _cwd: String = getcwd()?;
        println!("getcwd succeeded");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("getcwd error: {}", e.message.clone());
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
