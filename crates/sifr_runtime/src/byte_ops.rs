/// Count occurrences of one byte using the runtime's optimized byte search.
#[must_use]
pub fn count_byte(bytes: &[u8], needle: u8) -> usize {
    memchr::memchr_iter(needle, bytes).count()
}

#[cfg(test)]
mod tests {
    use super::count_byte;

    #[test]
    fn counts_matching_bytes() {
        assert_eq!(count_byte(b"banana", b'a'), 3);
        assert_eq!(count_byte(b"banana", b'z'), 0);
    }
}
