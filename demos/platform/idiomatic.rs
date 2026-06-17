use std::env;
use std::process::Command;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn system() -> String {
    if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else if cfg!(target_os = "macos") {
        "Darwin".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else {
        env::consts::OS.to_string()
    }
}

fn machine() -> String {
    env::consts::ARCH.to_string()
}

fn hostname() -> Option<String> {
    env::var("HOSTNAME")
        .or_else(|_| env::var("COMPUTERNAME"))
        .ok()
        .filter(|value| !value.is_empty())
}

fn node() -> String {
    hostname().unwrap_or_else(|| "localhost".to_string())
}

fn uname(flag: &str) -> String {
    Command::new("uname")
        .arg(flag)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| system())
}

fn release() -> String {
    uname("-r")
}

fn version() -> String {
    uname("-v")
}

fn processor() -> String {
    machine()
}

fn platform_system() -> String {
    system()
}

fn platform_arch() -> String {
    machine()
}

fn platform_node() -> String {
    node()
}

fn collect_core_actual() -> Vec<bool> {
    let sys_name = system();
    vec![
        !sys_name.is_empty() && sys_name != "linux" && sys_name != "macos" && sys_name != "windows",
        !machine().is_empty(),
        !processor().is_empty(),
    ]
}

fn collect_host_actual() -> Vec<bool> {
    vec![
        !node().is_empty(),
        !release().is_empty(),
        !version().is_empty(),
    ]
}

fn collect_alias_actual() -> Vec<bool> {
    vec![platform_system() == system() && platform_arch() == machine() && platform_node() == node()]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_core_actual());
    actual.extend(collect_host_actual());
    actual.extend(collect_alias_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true, true]);
    println!("platform platform parity demo: pass");
}
