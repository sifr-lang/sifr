fn smallest_or_zero(values: &Vec<i64>) -> i64 {
    let mut best: i64 = 9223372036854775807 as i64;
    for value in values.iter().copied() {
        if value < best {
            best = value;
        }
    }
    return if best != (9223372036854775807 as i64) {
        best
    } else {
        0 as i64
    };
}

fn main() {
    assert!(smallest_or_zero(&vec![8 as i64, 3 as i64, 7 as i64]) == (3 as i64));
    assert!(smallest_or_zero(&vec![]) == (0 as i64));
    println!("sentinel_values: ok");
}
