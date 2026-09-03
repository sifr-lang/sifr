// src/main.rs
use ::sifr_runtime::SifrInt;
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
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
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
        i = &i + &SifrInt::from_i64(1);
    }
}
fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let sys_name: String = system();
    let sifr_generated_chars_sys_name: Vec<char> = sys_name.chars().collect::<Vec<char>>();
    actual.push(
        &SifrInt::from(sifr_generated_chars_sys_name.len()) > &SifrInt::from_i64(0)
            && sys_name != "linux"
            && sys_name != "macos"
            && sys_name != "windows",
    );
    actual.push(&SifrInt::from(machine().chars().count()) > &SifrInt::from_i64(0));
    actual.push(&SifrInt::from(processor().chars().count()) > &SifrInt::from_i64(0));
    actual
}
fn collect_host_actual() -> Vec<bool> {
    vec![
        &SifrInt::from(node().chars().count()) > &SifrInt::from_i64(0),
        &SifrInt::from(release().chars().count()) > &SifrInt::from_i64(0),
        &SifrInt::from(version().chars().count()) > &SifrInt::from_i64(0),
    ]
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_core_actual());
    append_all(&mut actual, &collect_host_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("platform platform parity demo: pass");
}
