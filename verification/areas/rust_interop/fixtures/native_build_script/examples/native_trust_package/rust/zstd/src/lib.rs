pub fn artifact() -> String {
    include_str!(concat!(env!("OUT_DIR"), "/sifr-zstd-evidence.txt")).to_owned()
}

pub fn encode(input: &[u8]) -> Result<Vec<u8>, String> {
    zstd_upstream::stream::encode_all(input, 3).map_err(|error| error.to_string())
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, String> {
    zstd_upstream::stream::decode_all(input).map_err(|error| error.to_string())
}
