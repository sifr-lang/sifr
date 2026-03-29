fn gen_pairs(limit: i64) -> impl Iterator<Item = i64> {
    let mut current = 0_i64;
    std::iter::from_fn(move || {
        if current >= limit {
            return None;
        }

        let value = current;
        current += 1;
        Some(value)
    })
}

fn gen_even(values: impl IntoIterator<Item = i64>) -> impl Iterator<Item = i64> {
    values.into_iter().filter(|value| value % 2 == 0)
}

fn main() {
    let xs = [1_i64, 2, 3, 4, 5];

    let squares = xs
        .iter()
        .copied()
        .filter(|value| value % 2 == 0)
        .map(|value| value * value);

    println!("{:?}", squares.collect::<Vec<_>>());
    println!("{:?}", gen_pairs(5).collect::<Vec<_>>());
    println!("{:?}", gen_even(xs).collect::<Vec<_>>());
}
