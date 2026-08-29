fn main() {
    let width = std::hint::black_box(-1_i64);
    let _ = Vec::<u8>::with_capacity(width as usize);
}
