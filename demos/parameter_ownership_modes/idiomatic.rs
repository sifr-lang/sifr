fn borrowed_view(items: &[i64]) -> i64 {
    items.len() as i64
}

fn borrowed_mut_view(items: &mut [i64]) -> i64 {
    items.len() as i64
}

fn take_owned(items: Vec<i64>) -> Vec<i64> {
    items
}

fn take_owned_mutable(items: Vec<i64>) -> Vec<i64> {
    items
}

fn take_owned_mutable_reordered(items: Vec<i64>) -> Vec<i64> {
    items
}

fn main() {
    let values = vec![1_i64, 2, 3];
    let moved_once = take_owned(values);
    let moved_twice = take_owned_mutable(moved_once);
    let mut moved_thrice = take_owned_mutable_reordered(moved_twice);

    println!("{}", borrowed_view(&moved_thrice));
    println!("{}", borrowed_mut_view(&mut moved_thrice));
}
