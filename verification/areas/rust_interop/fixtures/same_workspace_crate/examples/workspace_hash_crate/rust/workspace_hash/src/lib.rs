pub fn hash(input: &[u8]) -> u64 {
    input.iter().fold(0xcbf2_9ce4_8422_2325, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

pub fn hash_pair(left: &[u8], right: &[u8]) -> u64 {
    let left_hash = hash(left);
    let right_hash = hash(right);
    left_hash.rotate_left(13) ^ right_hash.rotate_right(7)
}
