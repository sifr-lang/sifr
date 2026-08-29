fn main() {
    let value = 1_i64;
    let pointer = &value as *const i64;
    let _ = unsafe { *pointer };
}
