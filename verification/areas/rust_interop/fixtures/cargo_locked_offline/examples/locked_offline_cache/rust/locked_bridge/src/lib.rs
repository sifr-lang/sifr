pub fn cached_hash(input: &[u8]) -> u32 {
    input.iter().fold(0x811c_9dc5, |state, byte| {
        (state ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    })
}

pub fn lockfile_generation() -> u32 {
    3
}
