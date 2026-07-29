pub fn cached_hash(input: &[u8]) -> u32 {
    let mut indexed = indexmap::IndexMap::<String, u32>::new();
    for (offset, byte) in input.iter().copied().enumerate() {
        indexed.insert(offset.to_string(), u32::from(byte));
    }
    indexed.values().fold(0x811c_9dc5, |state, byte| {
        (state ^ *byte).wrapping_mul(0x0100_0193)
    })
}

pub fn lockfile_generation() -> u32 {
    2140
}
