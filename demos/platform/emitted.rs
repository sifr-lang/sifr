// src/main.rs
// --- stdlib: _sifr.platform ---
fn platform_system() -> String {
    ::sifr_stdlib::platform::platform_system()
}
fn platform_arch() -> String {
    ::sifr_stdlib::platform::platform_arch()
}
fn platform_node() -> String {
    ::sifr_stdlib::platform::platform_node()
}
fn platform_release() -> String {
    ::sifr_stdlib::platform::platform_release()
}
fn platform_version() -> String {
    ::sifr_stdlib::platform::platform_version()
}
fn platform_processor() -> String {
    ::sifr_stdlib::platform::platform_processor()
}

// --- stdlib: sifr.platform ---
fn system() -> String {
    platform_system()
}
fn machine() -> String {
    platform_arch()
}
fn node() -> String {
    platform_node()
}
fn release() -> String {
    platform_release()
}
fn version() -> String {
    platform_version()
}
fn processor() -> String {
    platform_processor()
}

// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
// --- end stdlib ---

fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let sys_name: String = system();
    let __sifr_chars_sys_name: Vec<char> = sys_name.chars().collect::<Vec<char>>();
    actual.push(((((sys_name.chars().count() as i64) > (0_i64)) && (sys_name != "linux")) && (sys_name != "macos")) && (sys_name != "windows"));
    actual.push((machine().chars().count() as i64) > (0_i64));
    actual.push((processor().chars().count() as i64) > (0_i64));
    actual
}

fn collect_host_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((node().chars().count() as i64) > (0_i64));
    actual.push((release().chars().count() as i64) > (0_i64));
    actual.push((version().chars().count() as i64) > (0_i64));
    actual
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_core_actual());
    append_all(&mut actual, &collect_host_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("platform platform parity demo: pass");
}
