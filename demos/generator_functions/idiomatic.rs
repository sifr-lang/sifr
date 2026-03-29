fn countdown(n: i64) -> impl Iterator<Item = i64> {
    (1..=n).rev()
}

fn format_optional(value: Option<i64>) -> String {
    value.map_or_else(|| "None".to_string(), |item| item.to_string())
}

fn main() {
    let mut values = countdown(3);
    let first = values.next();
    let second = values.next();
    let remaining: Vec<i64> = values.collect();

    let all_values: Vec<i64> = countdown(4).collect();

    assert_eq!(first, Some(3));
    assert_eq!(second, Some(2));
    assert_eq!(remaining, vec![1]);
    assert_eq!(all_values, vec![4, 3, 2, 1]);

    println!("{}", format_optional(first));
    println!("{}", format_optional(second));
    println!("{remaining:?}");
    println!("{all_values:?}");
}
