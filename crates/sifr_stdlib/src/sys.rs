use sifr_runtime::interop::SifrIntBridge;
use std::process::Command;

#[must_use]
pub fn env_get(key: &str) -> Option<String> {
    if !is_valid_env_key(key) {
        return None;
    }
    std::env::var(key).ok()
}

pub fn env_set(key: &str, value: &str) {
    if !is_valid_env_key(key) || value.as_bytes().contains(&0) {
        return;
    }
    std::env::set_var(key, value);
}

pub fn env_unset(key: &str) {
    if !is_valid_env_key(key) {
        return;
    }
    std::env::remove_var(key);
}

#[must_use]
pub fn env_keys() -> Vec<String> {
    std::env::vars_os()
        .map(|kv| kv.0.to_string_lossy().to_string())
        .collect()
}

#[must_use]
pub fn env_values() -> Vec<String> {
    std::env::vars_os()
        .map(|kv| kv.1.to_string_lossy().to_string())
        .collect()
}

#[must_use]
pub fn env_items() -> Vec<String> {
    std::env::vars_os()
        .map(|kv| format!("{}={}", kv.0.to_string_lossy(), kv.1.to_string_lossy()))
        .collect()
}

#[must_use]
pub fn get_args() -> Vec<String> {
    std::env::args().collect()
}

pub fn sys_exit(code: SifrIntBridge) {
    std::process::exit(code.to_i64_saturating() as i32);
}

#[must_use]
pub fn sys_version() -> String {
    "sifr 0.1.0".to_string()
}

#[must_use]
pub fn sys_platform() -> String {
    std::env::consts::OS.to_string()
}

#[must_use]
pub fn sys_maxsize() -> SifrIntBridge {
    SifrIntBridge::from(i64::MAX)
}

#[must_use]
pub fn getpid() -> SifrIntBridge {
    SifrIntBridge::from(i64::from(std::process::id()))
}

#[must_use]
pub fn cpu_count() -> SifrIntBridge {
    let count = std::thread::available_parallelism().map_or(1_i64, |parallelism| {
        i64::try_from(parallelism.get()).unwrap_or(i64::MAX)
    });
    SifrIntBridge::from(count)
}

#[must_use]
pub fn which(name: &str) -> Option<String> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
        .map(|path| path.to_string_lossy().to_string())
}

#[must_use]
pub fn os_sep() -> String {
    std::path::MAIN_SEPARATOR.to_string()
}

#[must_use]
pub fn os_linesep() -> String {
    if cfg!(target_os = "windows") {
        "\r\n".to_string()
    } else {
        "\n".to_string()
    }
}

#[must_use]
pub fn os_name() -> String {
    if cfg!(target_os = "windows") {
        "nt".to_string()
    } else {
        "posix".to_string()
    }
}

pub fn run_command(cmd: &str) -> Result<String, std::io::Error> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_valid_env_key(key: &str) -> bool {
    !key.is_empty() && !key.contains('=') && !key.as_bytes().contains(&0)
}

#[cfg(test)]
mod tests {
    use super::{
        cpu_count, env_get, env_items, env_keys, env_set, env_unset, env_values, get_args, getpid,
        os_linesep, os_name, os_sep, run_command, sys_maxsize, sys_platform, sys_version, which,
    };

    #[test]
    fn env_access_preserves_invalid_key_noop_policy() {
        env_set("", "x");
        env_set("A=B", "x");
        env_set("NUL\0KEY", "x");
        assert_eq!(env_get(""), None);
        assert_eq!(env_get("A=B"), None);
        assert_eq!(env_get("NUL\0KEY"), None);

        let key = "SIFR_STDLIB_SYS_TEST_VALUE";
        env_unset(key);
        env_set(key, "active");
        assert_eq!(env_get(key).as_deref(), Some("active"));
        assert!(env_keys().iter().any(|candidate| candidate == key));
        assert!(env_values().iter().any(|candidate| candidate == "active"));
        assert!(env_items()
            .iter()
            .any(|candidate| candidate == "SIFR_STDLIB_SYS_TEST_VALUE=active"));
        env_unset(key);
        assert_eq!(env_get(key), None);
    }

    #[test]
    fn sys_values_are_available_without_compiler_dispatch() {
        assert!(!get_args().is_empty());
        assert_eq!(sys_version(), "sifr 0.1.0");
        assert!(!sys_platform().is_empty());
        assert_eq!(sys_maxsize().to_i64_saturating(), i64::MAX);
        assert!(getpid().to_i64_saturating() > 0);
        assert!(cpu_count().to_i64_saturating() >= 1);
        assert!(!os_sep().is_empty());
        assert!(!os_linesep().is_empty());
        assert!(!os_name().is_empty());
        assert_eq!(which("__sifr_missing_tool_for_sys_test__"), None);
    }

    #[test]
    fn run_command_captures_trimmed_stdout() {
        assert_eq!(
            run_command("printf 'sifr\\n'").expect("command should run"),
            "sifr"
        );
    }
}
