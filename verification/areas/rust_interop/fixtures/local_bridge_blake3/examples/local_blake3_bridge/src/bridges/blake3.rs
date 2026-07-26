pub fn hash_bytes(input: &[u8]) -> Vec<u8> {
    blake3::hash(input).to_be_bytes().to_vec()
}

pub fn hash_hex(input: &[u8]) -> String {
    format!("{:016x}", blake3::hash(input))
}
