// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn platform_system() -> String {
        ::sifr_stdlib::platform::platform_system()
    }
    pub(super) fn platform_arch() -> String {
        ::sifr_stdlib::platform::platform_arch()
    }
    pub(super) fn platform_node() -> String {
        ::sifr_stdlib::platform::platform_node()
    }
    pub(super) fn platform_release() -> String {
        ::sifr_stdlib::platform::platform_release()
    }
    pub(super) fn platform_version() -> String {
        ::sifr_stdlib::platform::platform_version()
    }
    pub(super) fn platform_processor() -> String {
        ::sifr_stdlib::platform::platform_processor()
    }
    pub(super) fn system() -> String {
        platform_system()
    }
    pub(super) fn machine() -> String {
        platform_arch()
    }
    pub(super) fn node() -> String {
        platform_node()
    }
    pub(super) fn release() -> String {
        platform_release()
    }
    pub(super) fn version() -> String {
        platform_version()
    }
    pub(super) fn processor() -> String {
        platform_processor()
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
}
use crate::sifr_generated_generated_support::{
    assert_bool_vector_eq, machine, node, processor, release, system, version,
};
use ::sifr_runtime::SifrInt;
fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let sys_name: String = system();
    actual.push(
        sys_name.chars().count() > SifrInt::from_i64(0)
            && sys_name != "linux"
            && sys_name != "macos"
            && sys_name != "windows",
    );
    actual.push(machine().chars().count() > SifrInt::from_i64(0));
    actual.push(processor().chars().count() > SifrInt::from_i64(0));
    actual
}
fn collect_host_actual() -> Vec<bool> {
    vec![
        node().chars().count() > SifrInt::from_i64(0),
        release().chars().count() > SifrInt::from_i64(0),
        version().chars().count() > SifrInt::from_i64(0),
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
