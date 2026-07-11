#[must_use]
pub fn platform_system() -> String {
    if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else if cfg!(target_os = "macos") {
        "Darwin".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else {
        std::env::consts::OS.to_string()
    }
}

#[must_use]
pub fn platform_arch() -> String {
    std::env::consts::ARCH.to_string()
}

#[must_use]
pub fn platform_node() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}

#[must_use]
pub fn platform_release() -> String {
    uname_output("-r").unwrap_or_else(|| std::env::consts::OS.to_string())
}

#[must_use]
pub fn platform_version() -> String {
    uname_output("-v").unwrap_or_else(|| std::env::consts::OS.to_string())
}

#[must_use]
pub fn platform_processor() -> String {
    std::env::consts::ARCH.to_string()
}

fn uname_output(arg: &str) -> Option<String> {
    std::process::Command::new("uname")
        .arg(arg)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}
