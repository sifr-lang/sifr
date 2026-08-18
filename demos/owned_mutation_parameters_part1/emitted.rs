// src/main.rs
fn borrowed_view(items: &Vec<i64>) -> i64 {
    items.len() as i64
}

fn borrowed_mut_view(items: &mut Vec<i64>) -> i64 {
    items.len() as i64
}

fn take_owned(items: Vec<i64>) -> Vec<i64> {
    items
}

fn take_owned_mutable(mut items: Vec<i64>) -> Vec<i64> {
    items
}

fn take_owned_mutable_reordered(mut items: Vec<i64>) -> Vec<i64> {
    items
}

fn main() {
    let values: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    let moved_once: Vec<i64> = take_owned(values);
    let moved_twice: Vec<i64> = take_owned_mutable(moved_once);
    let mut moved_thrice: Vec<i64> = take_owned_mutable_reordered(moved_twice);
    println!("{}", borrowed_view(&moved_thrice));
    println!("{}", borrowed_mut_view(&mut moved_thrice));
}
