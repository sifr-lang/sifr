#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn sifr_cxx_probe_value() -> u32;
    }
}

fn sifr_cxx_probe_value() -> u32 {
    1_000_198
}

pub fn artifact() -> String {
    format!(
        "{};value={}",
        include_str!(concat!(env!("OUT_DIR"), "/sifr-cxx-evidence.txt")),
        sifr_cxx_probe_value()
    )
}
