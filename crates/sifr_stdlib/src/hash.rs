#[must_use]
pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha256 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn md5_bytes(data: &[u8]) -> Vec<u8> {
    md5::compute(data).0.to_vec()
}

#[must_use]
pub fn sha1_bytes(data: &[u8]) -> Vec<u8> {
    <sha1::Sha1 as sha1::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha224_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha224 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha384_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha384 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha512_bytes(data: &[u8]) -> Vec<u8> {
    <sha2_0_11::Sha512 as sha2_0_11::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
    <blake2::Blake2b512 as blake2::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
    <blake2::Blake2s256 as blake2::Digest>::digest(data).to_vec()
}
