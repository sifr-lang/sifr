//! Check the library that supplied CPython symbols before initializing it.
use std::path::Path;

/// A same-basename shared library selected by the process loader is not
/// necessarily the interpreter used to certify the generated program.
#[allow(unsafe_code)]
pub fn validate_loaded_library(expected: &str, expected_sha256: &str) -> Result<(), String> {
    #[cfg(unix)]
    {
        let actual = loaded_library()?;
        let expected_path = Path::new(expected).canonicalize().map_err(|error| {
            format!("selected Python library {expected:?} is unavailable: {error}")
        })?;
        let actual_path = Path::new(&actual)
            .canonicalize()
            .map_err(|error| format!("loaded Python library {actual:?} is unavailable: {error}"))?;
        if actual_path != expected_path {
            return Err(format!(
                "Python loader mismatch: selected {}, loaded {}",
                expected_path.display(),
                actual_path.display(),
            ));
        }
        let bytes = std::fs::read(&actual_path)
            .map_err(|error| format!("cannot read loaded Python library: {error}"))?;
        let actual_sha256 = library_digest(&bytes);
        if actual_sha256 != expected_sha256 {
            return Err(format!(
                "Python loader mismatch: selected library content changed at {}",
                actual_path.display()
            ));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (expected, expected_sha256);
        Err("Python shared-library qualification is unsupported on this target".to_owned())
    }
}

fn library_digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        let _ = write!(result, "{byte:02x}");
    }
    result
}

#[cfg(unix)]
#[allow(unsafe_code)]
fn loaded_library() -> Result<String, String> {
    let mut info = std::mem::MaybeUninit::<libc::Dl_info>::uninit();
    // SAFETY: Py_GetVersion is a linked function; dladdr writes a Dl_info
    // on success, and its filename is a loader-owned terminated string.
    let actual = unsafe {
        if libc::dladdr(
            pyo3::ffi::Py_GetVersion as *const () as *const _,
            info.as_mut_ptr(),
        ) == 0
        {
            return Err("cannot identify the loaded Python shared library".to_owned());
        }
        let info = info.assume_init();
        if info.dli_fname.is_null() {
            return Err("loaded Python shared library has no filename".to_owned());
        }
        std::ffi::CStr::from_ptr(info.dli_fname)
            .to_string_lossy()
            .into_owned()
    };

    Ok(actual)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn dx9_python_loader_rejects_real_alternate_same_basename_library() {
        const CHILD: &str = "SIFR_DX9_LOADER_CHILD";
        if let Ok(expected) = std::env::var(CHILD) {
            let error = validate_loaded_library(
                &expected,
                &std::env::var("SIFR_DX9_LOADER_SHA").expect("selected digest"),
            )
            .expect_err("wrong library must fail");
            assert!(error.contains("Python loader mismatch"), "{error}");
            return;
        }
        let actual = Path::new(&loaded_library().expect("loaded CPython"))
            .canonicalize()
            .expect("library path");
        let digest = library_digest(&std::fs::read(&actual).expect("library bytes"));
        validate_loaded_library(actual.to_str().expect("library UTF8"), &digest)
            .expect("selected loader");
        assert!(
            validate_loaded_library(actual.to_str().expect("library UTF8"), &"0".repeat(64))
                .expect_err("changed same-path library content")
                .contains("content changed")
        );
        let directory =
            std::env::temp_dir().join(format!("sifr-dx9-loader-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("fixture directory");
        let copy = directory.join(actual.file_name().expect("library basename"));
        std::fs::copy(&actual, &copy).expect("copy actual CPython library");
        // SONAME can omit the final patch suffix. Read it from the loaded ELF.
        let output = std::process::Command::new("readelf")
            .args(["-d"])
            .arg(&actual)
            .output()
            .expect("read ELF dynamic section");
        assert!(output.status.success());
        let dynamic = String::from_utf8_lossy(&output.stdout);
        let soname = dynamic
            .lines()
            .find(|line| line.contains("(SONAME)"))
            .and_then(|line| line.split('[').nth(1))
            .and_then(|text| text.split(']').next())
            .expect("CPython SONAME");
        if directory.join(soname) != copy {
            std::os::unix::fs::symlink(copy.file_name().expect("basename"), directory.join(soname))
                .expect("SONAME alias");
        }
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["python::loader::tests::dx9_python_loader_rejects_real_alternate_same_basename_library", "--exact", "--nocapture"])
            .env(CHILD, &actual)
            .env("SIFR_DX9_LOADER_SHA", &digest)
            .env("LD_LIBRARY_PATH", format!("{}:{}", directory.display(), std::env::var("LD_LIBRARY_PATH").unwrap_or_default()))
            .output().expect("alternate loader process");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::remove_dir_all(directory).expect("cleanup");
    }
}
