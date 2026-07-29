include!(concat!(env!("OUT_DIR"), "/sifr.probe.rs"));

pub fn schema_version() -> u64 {
    Probe {
        id: 1404,
        payload: Vec::new(),
    }
    .id
}

pub fn generated_artifact() -> String {
    include_str!(concat!(env!("OUT_DIR"), "/sifr-prost-build-evidence.txt")).to_owned()
}
