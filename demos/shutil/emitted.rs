// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::IOError;
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn read_text(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(super) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(super) fn mkdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn rename(src: &str, dst: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rename(src, dst).map_err(sifr_generated_io_err)
    }
    pub(super) fn disk_usage(path: &str) -> Vec<SifrInt> {
        ::sifr_stdlib::fs::disk_usage(path)
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(super) fn copy_file(src: &str, dst: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::copy_file(src, dst).map_err(sifr_generated_io_err)
    }
    pub(super) fn rmdir_all(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir_all(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub(super) fn run_command(cmd: &str) -> Result<String, IOError> {
        ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
    }
    pub(super) fn which(name: &str) -> Option<String> {
        ::sifr_stdlib::sys::which(name)
    }
    pub(super) fn copy(src: &str, dst: &str) -> Result<(), IOError> {
        copy_file(src, dst)
    }
    pub(super) fn move_file(src: &str, dst: &str) -> Result<(), IOError> {
        rename(src, dst)
    }
    pub(super) fn rmtree(path: &str) -> Result<(), IOError> {
        rmdir_all(path)
    }
    pub(super) fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
        ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
    }
    pub(super) fn sifr_generated_random_suffix() -> String {
        let n: SifrInt = random_int(SifrInt::from_i64(100_000), SifrInt::from_i64(999_999));
        n.to_string()
    }
    pub(super) fn mktemp_path(prefix: &str) -> String {
        let suffix: String = sifr_generated_random_suffix();
        let mut root: String = gettempdir();
        let sifr_generated_chars_root: Vec<char> = root.chars().collect::<Vec<char>>();
        if sifr_generated_chars_root.len() == SifrInt::from_i64(0) {
            root = "/tmp".to_string();
        } else {
            let last: Option<String> = {
                let sifr_generated_string_index =
                    ::std::ops::Sub::sub(SifrInt::from(root.chars().count()), SifrInt::from_i64(1));
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_root.len());
                sifr_generated_chars_root
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(last) = last
                && last == "/"
            {
                return {
                    let mut sifr_generated_concat: String = String::with_capacity(
                        root.len()
                            .saturating_add(prefix.len())
                            .saturating_add(suffix.len()),
                    );
                    sifr_generated_concat.push_str(root.as_str());
                    sifr_generated_concat.push_str(prefix);
                    sifr_generated_concat.push_str(suffix.as_str());
                    sifr_generated_concat
                };
            }
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(
                root.len()
                    .saturating_add(1usize)
                    .saturating_add(prefix.len())
                    .saturating_add(suffix.len()),
            );
            sifr_generated_concat.push_str(root.as_str());
            sifr_generated_concat.push('/');
            sifr_generated_concat.push_str(prefix);
            sifr_generated_concat.push_str(suffix.as_str());
            sifr_generated_concat
        }
    }
    pub(super) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
            assert_eq!(
                {
                    let sifr_generated_condition_list = &actual;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                }
            );
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
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
    assert_bool_vector_eq, copy, disk_usage, exists, mkdir, mktemp_path, move_file, read_text,
    rmtree, run_command, which, write_text,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
fn collect_copy_move_tree_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let base: String = mktemp_path("sifr_shutil_shutil_demo_");
    let src: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(base.len().saturating_add(8usize));
        sifr_generated_concat.push_str(base.as_str());
        sifr_generated_concat.push_str("/src.txt");
        sifr_generated_concat
    };
    let copied: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(base.len().saturating_add(11usize));
        sifr_generated_concat.push_str(base.as_str());
        sifr_generated_concat.push_str("/copied.txt");
        sifr_generated_concat
    };
    let moved: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(base.len().saturating_add(10usize));
        sifr_generated_concat.push_str(base.as_str());
        sifr_generated_concat.push_str("/moved.txt");
        sifr_generated_concat
    };
    let tree: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(base.len().saturating_add(5usize));
        sifr_generated_concat.push_str(base.as_str());
        sifr_generated_concat.push_str("/tree");
        sifr_generated_concat
    };
    let nested: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(tree.len().saturating_add(11usize));
        sifr_generated_concat.push_str(tree.as_str());
        sifr_generated_concat.push_str("/nested.txt");
        sifr_generated_concat
    };
    let mut copy_ok: bool = false;
    let mut move_ok_value_16a723bbd15dd243: bool = false;
    let mut rmtree_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        mkdir(&base)?;
        write_text(&src, "demo")?;
        copy(&src, &copied)?;
        let mut copied_content_ok: bool = false;
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let copied_content: String = read_text(&copied)?;
            copied_content_ok = copied_content == "demo";
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err;
            let _ = e.message;
        }
        copy_ok = exists(&src) && exists(&copied) && copied_content_ok;
        move_file(&copied, &moved)?;
        move_ok_value_16a723bbd15dd243 = exists(&moved) && !exists(&copied);
        mkdir(&tree)?;
        write_text(&nested, "nested")?;
        rmtree(&tree)?;
        rmtree_ok = !exists(&tree);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
    }
    actual.push(copy_ok);
    actual.push(move_ok_value_16a723bbd15dd243);
    actual.push(rmtree_ok);
    actual
}
fn collect_tooling_and_cleanup_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let base: String = mktemp_path("sifr_shutil_shutil_demo_cleanup_");
    let mut base_ready: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        mkdir(&base)?;
        base_ready = exists(&base);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
    }
    let mut which_ok: bool = false;
    let tool: Option<String> = which("sh");
    if let Some(tool) = tool {
        which_ok = tool.chars().count() > SifrInt::from_i64(0);
    }
    actual.push(which_ok);
    let usage: Vec<SifrInt> = disk_usage(&base.to_string());
    let mut usage_ok: bool = false;
    if usage.len() == SifrInt::from_i64(3) {
        let total: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &usage;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if let Some(total) = total {
            usage_ok = total > SifrInt::from_i64(0);
        }
    }
    usage_ok = usage_ok && base_ready;
    actual.push(usage_ok);
    let mut missing_copy_rejected: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        copy(
            &format!("{base}/missing_src.txt"),
            &format!("{base}/missing_dst.txt"),
        )?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
        missing_copy_rejected = true;
    }
    actual.push(missing_copy_rejected);
    let mut cleanup_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _ = run_command(&format!("rm -rf {base}"))?;
        cleanup_ok = !exists(&base);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
        cleanup_ok = !exists(&base);
    }
    actual.push(cleanup_ok);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_copy_move_tree_actual());
    append_all(&mut actual, &collect_tooling_and_cleanup_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("shutil shutil parity demo: pass");
}
