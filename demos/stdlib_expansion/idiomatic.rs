use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ASCII_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";

fn bisect_left(values: &[i64], target: i64) -> usize {
    values.partition_point(|value| *value < target)
}

fn reduce<F>(values: &[i64], initial: i64, reducer: F) -> i64
where
    F: Fn(i64, i64) -> i64,
{
    values.iter().copied().fold(initial, reducer)
}

fn token_hex(byte_len: usize) -> String {
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64;
    let mut bytes = Vec::with_capacity(byte_len);
    for _ in 0..byte_len {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        bytes.push((seed & 0xff) as u8);
    }
    bytes
        .into_iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn mean(values: &[f64]) -> Result<f64, String> {
    if values.is_empty() {
        return Err("mean requires at least one value".to_string());
    }
    Ok(values.iter().sum::<f64>() / values.len() as f64)
}

fn take<T: Clone>(count: usize, values: &[T]) -> Vec<T> {
    values.iter().take(count).cloned().collect()
}

fn fill(text: &str, width: usize) -> Result<String, String> {
    if width == 0 {
        return Err("width must be positive".to_string());
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if next_len > width && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    Ok(lines.join("\n"))
}

fn parse_row(line: &str) -> Vec<String> {
    line.split(',').map(str::to_string).collect()
}

fn format_row(values: &[&str]) -> String {
    values.join(",")
}

fn parse_flag(args: &[&str], flag: &str) -> bool {
    args.iter().any(|arg| *arg == flag)
}

fn parse_option(args: &[&str], name: &str, default: &str) -> String {
    for (index, arg) in args.iter().enumerate() {
        if *arg == name {
            if let Some(value) = args.get(index + 1) {
                if !value.starts_with("--") {
                    return (*value).to_string();
                }
            }
        }
    }
    default.to_string()
}

fn fnmatch(name: &str, pattern: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix("*.") {
        return name.rsplit('.').next() == Some(suffix);
    }
    name == pattern
}

fn main() {
    assert_eq!(ASCII_LOWERCASE.len(), 26);
    assert_eq!(DIGITS.len(), 10);

    let data = [10_i64, 20, 30, 40, 50];
    assert_eq!(bisect_left(&data, 30), 2);

    let nums = [1_i64, 2, 3, 4, 5];
    let total = reduce(&nums, 0, |left, right| left + right);
    assert_eq!(total, 15);

    let token = token_hex(8);
    assert_eq!(token.len(), 16);

    let stats_values = [2.0_f64, 4.0, 6.0];
    match mean(&stats_values) {
        Ok(avg) => assert_eq!(avg, 4.0),
        Err(message) => panic!("unexpected statistics failure: {}", message),
    }

    let mut heap = BinaryHeap::new();
    heap.push(Reverse(5_i64));
    heap.push(Reverse(1_i64));
    heap.push(Reverse(3_i64));
    assert_eq!(heap.pop(), Some(Reverse(1)));

    let mut small_heap = BinaryHeap::new();
    for value in [9_i64, 3, 7, 1] {
        small_heap.push(Reverse(value));
    }
    let mut smallest = Vec::new();
    for _ in 0..2 {
        if let Some(Reverse(value)) = small_heap.pop() {
            smallest.push(value);
        }
    }
    assert_eq!(smallest.len(), 2);

    let merged = [1_i64, 2]
        .iter()
        .copied()
        .chain([3_i64, 4].iter().copied())
        .collect::<Vec<_>>();
    assert_eq!(merged.len(), 4);
    let first_three = take(3, &[10_i64, 20, 30, 40]);
    assert_eq!(first_three.len(), 3);

    match fill("hello world foo bar", 12) {
        Ok(filled) => assert!(!filled.is_empty()),
        Err(message) => panic!("unexpected fill failure: {}", message),
    }

    let row = parse_row("a,b,c");
    assert_eq!(row.len(), 3);
    let line = format_row(&["x", "y"]);
    assert_eq!(line, "x,y");

    let args = ["--output", "file.txt"];
    assert!(parse_flag(&args, "--output"));
    assert_eq!(parse_option(&args, "--output", "default"), "file.txt");

    assert!(fnmatch("test.py", "*.py"));
    assert!(!fnmatch("test.py", "*.txt"));

    println!("stdlib_expansion demo: all checks passed!");
}
