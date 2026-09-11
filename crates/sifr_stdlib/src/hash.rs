#[must_use]
pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    <sha2::Sha256 as sha2::Digest>::digest(data).to_vec()
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
    <sha2::Sha224 as sha2::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha384_bytes(data: &[u8]) -> Vec<u8> {
    <sha2::Sha384 as sha2::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn sha512_bytes(data: &[u8]) -> Vec<u8> {
    <sha2::Sha512 as sha2::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
    <blake2::Blake2b512 as blake2::Digest>::digest(data).to_vec()
}

#[must_use]
pub fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
    <blake2::Blake2s256 as blake2::Digest>::digest(data).to_vec()
}

#[cfg(test)]
mod tests {
    use super::{blake2b_bytes, blake2s_bytes};

    #[test]
    fn blake2_digests_match_cpython_vectors() {
        // CPython 3.14.7 hashlib.blake2b/blake2s, with their default digest sizes.
        let cases: [(&[u8], &str, &str); 2] = [
            (
                b"",
                "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce",
                "69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9",
            ),
            (
                b"abc",
                "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923",
                "508c5e8c327c14e2e1a72ba34eeb452f37458b209ed63a294d999b4c86675982",
            ),
        ];
        for (input, expected_b, expected_s) in cases {
            for (digest, expected) in [
                (blake2b_bytes(input), expected_b),
                (blake2s_bytes(input), expected_s),
            ] {
                let hexadecimal: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
                assert_eq!(hexadecimal, expected);
            }
        }
    }
}
