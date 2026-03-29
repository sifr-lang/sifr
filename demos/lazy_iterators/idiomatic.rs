fn fibonacci(n: usize) -> impl Iterator<Item = i64> {
    let mut a = 0_i64;
    let mut b = 1_i64;
    let mut count = 0_usize;
    std::iter::from_fn(move || {
        if count >= n {
            return None;
        }
        let current = a;
        let next = a + b;
        a = b;
        b = next;
        count += 1;
        Some(current)
    })
}

fn squares(n: usize) -> impl Iterator<Item = i64> {
    (0..n).map(|i| {
        let value = i as i64;
        value * value
    })
}

fn evens(limit: usize) -> impl Iterator<Item = i64> {
    (0..limit).filter(|i| i % 2 == 0).map(|i| i as i64)
}

fn count_up(n: usize) -> impl Iterator<Item = i64> {
    (0..n).map(|i| i as i64)
}

fn format_int_list(values: &[i64]) -> String {
    let joined = values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

fn main() {
    let mut output = Vec::new();

    output.push("=== Fibonacci (lazy for loop) ===".to_string());
    for fib in fibonacci(8) {
        output.push(fib.to_string());
    }

    output.push("=== Squares (collected) ===".to_string());
    let sq: Vec<i64> = squares(5).collect();
    output.push(format_int_list(&sq));

    output.push("=== Evens (conditional yield) ===".to_string());
    for even in evens(10) {
        output.push(even.to_string());
    }

    output.push("=== Count (lazy) ===".to_string());
    for value in count_up(3) {
        output.push(value.to_string());
    }

    output.push("=== Count (collected) ===".to_string());
    let nums: Vec<i64> = count_up(5).collect();
    output.push(format_int_list(&nums));

    assert_eq!(
        output,
        vec![
            "=== Fibonacci (lazy for loop) ===",
            "0",
            "1",
            "1",
            "2",
            "3",
            "5",
            "8",
            "13",
            "=== Squares (collected) ===",
            "[0, 1, 4, 9, 16]",
            "=== Evens (conditional yield) ===",
            "0",
            "2",
            "4",
            "6",
            "8",
            "=== Count (lazy) ===",
            "0",
            "1",
            "2",
            "=== Count (collected) ===",
            "[0, 1, 2, 3, 4]",
        ]
    );

    println!("Lazy iterator demo output:");
    for item in output {
        println!("{item}");
    }
}
