fn main() {
    let value = std::hint::black_box(1_i64);
    let _ = value + 1;
}
