pub fn digest(input: &[u8]) -> Vec<u8> {
    digest_u64(input).to_be_bytes().to_vec()
}

pub fn digest_hex(input: &[u8]) -> String {
    // A shared bridge crate may mention crate::__sifr_bridge in comments, but
    // must not import package-generated bridge modules.
    format!("{:016x}", digest_u64(input))
}

fn digest_u64(input: &[u8]) -> u64 {
    input.iter().fold(0xcbf2_9ce4_8422_2325, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
