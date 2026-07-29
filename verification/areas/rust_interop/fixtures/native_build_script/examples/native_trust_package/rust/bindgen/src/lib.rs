pub fn artifact() -> String {
    include_str!(concat!(env!("OUT_DIR"), "/sifr-bindgen-evidence.txt")).to_owned()
}
