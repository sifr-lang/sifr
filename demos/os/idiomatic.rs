use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

type IOError = io::Error;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn run_command(command: &str) -> Result<String, IOError> {
    let output = Command::new("sh").arg("-c").arg(command).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

fn mkdir(path: &str) -> Result<(), IOError> {
    fs::create_dir(path)
}

fn listdir(path: &str) -> Result<Vec<String>, IOError> {
    let mut entries = fs::read_dir(path)?
        .map(|entry| entry.map(|item| item.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

fn rmdir(path: &str) -> Result<(), IOError> {
    fs::remove_dir(path)
}

fn getpid() -> u32 {
    std::process::id()
}

fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

fn remove_file(path: &str) -> Result<(), IOError> {
    fs::remove_file(path)
}

fn stat(path: &str) -> Result<u64, IOError> {
    Ok(fs::metadata(path)?.len())
}

fn collect_runtime_actual() -> Vec<bool> {
    vec![run_command("echo sifr_os_demo")
        .map(|output| output == "sifr_os_demo")
        .unwrap_or(false)]
}

fn collect_filesystem_actual() -> Vec<bool> {
    let base = format!("/tmp/sifr_os_os_demo_{}", getpid());
    let file_path = format!("{base}/demo.txt");

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_dir(&base);

    let (os_flow_ok, list_ok, stat_ok, cleanup_ok) =
        (|| -> Result<(bool, bool, bool, bool), IOError> {
            mkdir(&base)?;
            fs::write(&file_path, "demo")?;

            let os_flow_ok = is_dir(&base) && is_file(&file_path);
            let list_ok = !listdir(&base)?.is_empty();
            let stat_ok = stat(&file_path)? > 0;

            remove_file(&file_path)?;
            rmdir(&base)?;

            Ok((os_flow_ok, list_ok, stat_ok, !is_dir(&base)))
        })()
        .unwrap_or((false, false, false, false));

    vec![os_flow_ok, list_ok, stat_ok, cleanup_ok]
}

fn collect_missing_actual() -> Vec<bool> {
    vec![rmdir("/tmp/sifr_os_os_demo_missing").is_err()]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_runtime_actual());
    actual.extend(collect_filesystem_actual());
    actual.extend(collect_missing_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("os os parity demo: pass");
}
