use std::collections::hash_map::RandomState;

pub fn cached_hash(input: &[u8]) -> u32 {
    let mut indexed =
        indexmap::IndexMap::<String, u32, RandomState>::with_hasher(RandomState::default());
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

#[cfg(test)]
mod tests {
    use super::{cached_hash, lockfile_generation};

    #[test]
    fn cached_hash_preserves_input_order_and_values() {
        assert_eq!(cached_hash(b""), 0x811c_9dc5);
        assert_eq!(cached_hash(b"sifr-rust-interop"), 0xdc36_289a);
        assert_eq!(cached_hash(&[0, 255, 1]), 0xf38f_4273);
        assert_ne!(cached_hash(b"ab"), cached_hash(b"ba"));
        assert_eq!(lockfile_generation(), 2140);
    }
}
