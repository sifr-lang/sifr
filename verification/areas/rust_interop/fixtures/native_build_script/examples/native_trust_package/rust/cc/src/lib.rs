pub fn artifact() -> String {
    include_str!(concat!(env!("OUT_DIR"), "/sifr-cc-evidence.txt")).to_owned()
}
