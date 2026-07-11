#[must_use]
pub fn sha256(s: &str) -> String {
    hex_digest(sha256_bytes(s.as_bytes()))
}

#[must_use]
pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha256 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn md5(s: &str) -> String {
    hex_digest(md5_bytes(s.as_bytes()))
}

#[must_use]
pub fn md5_bytes(data: &[u8]) -> Vec<u8> {
    md5::compute(data).0.to_vec()
}

#[must_use]
pub fn sha1(s: &str) -> String {
    hex_digest(sha1_bytes(s.as_bytes()))
}

#[must_use]
pub fn sha1_bytes(data: &[u8]) -> Vec<u8> {
    <sha1::Sha1 as sha1::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha224(s: &str) -> String {
    hex_digest(sha224_bytes(s.as_bytes()))
}

#[must_use]
pub fn sha224_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha224 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha384(s: &str) -> String {
    hex_digest(sha384_bytes(s.as_bytes()))
}

#[must_use]
pub fn sha384_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha384 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha512(s: &str) -> String {
    hex_digest(sha512_bytes(s.as_bytes()))
}

#[must_use]
pub fn sha512_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha512 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn blake2b(s: &str) -> String {
    hex_digest(blake2b_bytes(s.as_bytes()))
}

#[must_use]
pub fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
    <blake2::Blake2b512 as blake2::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn blake2s(s: &str) -> String {
    hex_digest(blake2s_bytes(s.as_bytes()))
}

#[must_use]
pub fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
    <blake2::Blake2s256 as blake2::Digest>::digest(data).to_vec()
}

fn hex_digest(bytes: Vec<u8>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}
