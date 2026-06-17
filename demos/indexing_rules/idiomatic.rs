fn normalize_index(len: usize, index: i64) -> Option<usize> {
    let len = len as i64;
    let normalized = if index < 0 { len + index } else { index };
    ((0..len).contains(&normalized)).then_some(normalized as usize)
}

fn remove_at<T>(items: &mut Vec<T>, index: i64) {
    if let Some(index) = normalize_index(items.len(), index) {
        let _ = items.remove(index);
    }
}

fn main() {
    let mut items = vec![1_i64, 2, 3];

    if let Some(index) = normalize_index(items.len(), -1) {
        items[index] = 9;
    }
    if let Some(index) = normalize_index(items.len(), -2) {
        items[index] += 5;
    }
    remove_at(&mut items, -1);
    remove_at(&mut items, -5);

    println!("indexing_rules indexing and semantics parity fixes demo:");
    println!("{items:?}");
}
