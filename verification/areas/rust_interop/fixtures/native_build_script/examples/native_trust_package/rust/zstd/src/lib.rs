pub fn encode(input: &[u8]) -> Vec<u8> {
    input.iter().copied().rev().collect()
}
