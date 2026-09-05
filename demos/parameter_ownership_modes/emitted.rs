// src/main.rs
use ::sifr_runtime::SifrInt;
fn borrowed_view(items: &[SifrInt]) -> SifrInt {
    SifrInt::from(items.len())
}
fn borrowed_mut_view(items: &mut Vec<SifrInt>) -> SifrInt {
    SifrInt::from(items.len())
}
const fn take_owned(items: Vec<SifrInt>) -> Vec<SifrInt> {
    items
}
const fn take_owned_mutable(items: Vec<SifrInt>) -> Vec<SifrInt> {
    items
}
const fn take_owned_mutable_reordered(items: Vec<SifrInt>) -> Vec<SifrInt> {
    items
}
fn main() {
    let values: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let moved_once: Vec<SifrInt> = take_owned(values);
    let moved_twice: Vec<SifrInt> = take_owned_mutable(moved_once);
    let mut moved_thrice: Vec<SifrInt> = take_owned_mutable_reordered(moved_twice);
    println!("{}", borrowed_view(&moved_thrice));
    println!("{}", borrowed_mut_view(&mut moved_thrice));
}
