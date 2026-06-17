use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

type IOError = io::Error;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    fs::write(path, content)
}

fn read_text(path: &str) -> Result<String, IOError> {
    fs::read_to_string(path)
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn mkdir(path: &str) -> Result<(), IOError> {
    fs::create_dir_all(path)
}

fn mktemp_path(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir()
        .join(format!("{prefix}{nanos}-{}-{counter}", std::process::id()))
        .to_string_lossy()
        .into_owned()
}

fn copy(src: &str, dst: &str) -> Result<(), IOError> {
    fs::copy(src, dst).map(|_| ())
}

fn move_file(src: &str, dst: &str) -> Result<(), IOError> {
    fs::rename(src, dst)
}

fn rmtree(path: &str) -> Result<(), IOError> {
    fs::remove_dir_all(path)
}

fn which(name: &str) -> Option<String> {
    let candidate = Path::new(name);
    if candidate.components().count() > 1 && is_executable(candidate) {
        return Some(candidate.to_string_lossy().into_owned());
    }

    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|directory| directory.join(name))
            .find(|path| is_executable(path))
            .map(|path| path.to_string_lossy().into_owned())
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && path
            .metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn disk_usage(path: &str) -> Vec<u64> {
    let path = PathBuf::from(path);
    let total = fs2::total_space(&path).unwrap_or(0);
    let free = fs2::available_space(&path).unwrap_or(0);
    vec![total, total.saturating_sub(free), free]
}

fn collect_copy_move_tree_actual() -> Vec<bool> {
    let base = mktemp_path("sifr_shutil_shutil_demo_");
    let _ = fs::remove_dir_all(&base);

    let src = format!("{base}/src.txt");
    let copied = format!("{base}/copied.txt");
    let moved = format!("{base}/moved.txt");
    let tree = format!("{base}/tree");
    let nested = format!("{tree}/nested.txt");

    let (copy_ok, move_ok, rmtree_ok) = (|| -> Result<(bool, bool, bool), IOError> {
        mkdir(&base)?;
        write_text(&src, "demo")?;
        copy(&src, &copied)?;
        let copy_ok = exists(&src) && exists(&copied) && read_text(&copied)? == "demo";

        move_file(&copied, &moved)?;
        let move_ok = exists(&moved) && !exists(&copied);

        mkdir(&tree)?;
        write_text(&nested, "nested")?;
        rmtree(&tree)?;
        Ok((copy_ok, move_ok, !exists(&tree)))
    })()
    .unwrap_or((false, false, false));

    vec![copy_ok, move_ok, rmtree_ok]
}

fn collect_tooling_and_cleanup_actual() -> Vec<bool> {
    let base = mktemp_path("sifr_shutil_shutil_demo_cleanup_");
    let _ = fs::remove_dir_all(&base);
    let base_ready = mkdir(&base).map(|_| exists(&base)).unwrap_or(false);

    let which_ok = which("sh").is_some_and(|tool| !tool.is_empty());
    let usage = disk_usage(&base);
    let usage_ok = base_ready && usage.len() == 3 && usage[0] > 0;
    let missing_copy_rejected = copy(
        &format!("{base}/missing_src.txt"),
        &format!("{base}/missing_dst.txt"),
    )
    .is_err();

    let cleanup_ok = fs::remove_dir_all(&base).is_ok() && !exists(&base);

    vec![which_ok, usage_ok, missing_copy_rejected, cleanup_ok]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_copy_move_tree_actual());
    actual.extend(collect_tooling_and_cleanup_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true, true]);
    println!("shutil shutil parity demo: pass");
}
