fn index_of<T: PartialEq>(items: &[T], needle: &T) -> Option<usize> {
    items.iter().position(|item| item == needle)
}

fn main() {
    let mut items = vec![10_i64, 20, 30];
    if let Some(index) = index_of(&items, &99) {
        items.remove(index);
    }
    println!("After removing missing 99:");
    println!("{items:?}");

    if let Some(index) = index_of(&items, &20) {
        items.remove(index);
    }
    println!("After removing 20:");
    println!("{items:?}");

    let names = vec![
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
    ];
    if let Some(pos) = index_of(&names, &"bob".to_string()) {
        println!("Found 'bob' at index {pos}");
    } else {
        println!("'bob' not found");
    }
    if let Some(pos) = index_of(&names, &"dave".to_string()) {
        println!("Found 'dave' at index {pos}");
    } else {
        println!("'dave' not found (safe: returned None)");
    }

    let nums = vec![5_i64, 3, 8, 1, 9];
    if let (Some(lo), Some(hi)) = (nums.iter().copied().min(), nums.iter().copied().max()) {
        println!("min={lo}, max={hi}");
    }

    let empty: Vec<i64> = vec![];
    if empty.iter().copied().min().is_none() {
        println!("min([]) = None (safe!)");
    }
    if empty.iter().copied().max().is_none() {
        println!("max([]) = None (safe!)");
    }

    let floats = vec![3.14_f64, 1.0, 2.71, 0.5];
    let mut sorted_floats = floats.clone();
    sorted_floats.sort_by(f64::total_cmp);
    println!("sorted floats:");
    println!("{sorted_floats:?}");

    let mut stack = vec![42_i64];
    let val1 = stack.pop();
    let val2 = stack.pop();
    if let Some(value) = val1 {
        println!("popped: {value}");
    }
    if val2.is_none() {
        println!("pop on empty = None (safe!)");
    }

    println!("min(3, 7) = {}", std::cmp::min(3_i64, 7));
    println!("max(3, 7) = {}", std::cmp::max(3_i64, 7));
    println!();
    println!("All collection operations are panic-free!");
    println!("  - list.remove(missing) -> no-op");
    println!("  - list.index(missing) -> None");
    println!("  - min/max(empty) -> None");
    println!("  - sorted(floats) -> total_cmp (NaN-safe)");
    println!("  - list.pop(empty) -> None");
}
