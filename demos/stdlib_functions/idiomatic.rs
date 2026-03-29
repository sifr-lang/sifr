use std::cmp::Ordering;

fn factorial(n: i64) -> i64 {
    if n < 0 {
        return 0;
    }
    (1..=n).product::<i64>().max(1)
}

fn gcd(a: i64, b: i64) -> i64 {
    let mut left = a.abs();
    let mut right = b.abs();
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a.abs() / gcd(a, b)) * b.abs()
    }
}

fn comb(n: i64, k: i64) -> i64 {
    if k < 0 || k > n {
        return 0;
    }
    let r = k.min(n - k);
    let mut result = 1_i64;
    for step in 0..r {
        result = result * (n - step) / (step + 1);
    }
    result
}

fn perm(n: i64, k: i64) -> i64 {
    if k < 0 || k > n {
        return 0;
    }
    (0..k).fold(1_i64, |acc, step| acc * (n - step))
}

fn prod(values: &[i64]) -> i64 {
    values.iter().product()
}

fn exp(value: f64) -> f64 {
    value.exp()
}

fn isfinite(value: f64) -> bool {
    value.is_finite()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatisticsError {
    message: String,
}

impl StatisticsError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

fn harmonic_mean(values: &[f64]) -> Result<f64, StatisticsError> {
    if values.is_empty() {
        return Err(StatisticsError::new(
            "harmonic_mean requires at least one data point",
        ));
    }

    let mut reciprocal_total = 0.0;
    for value in values {
        if *value <= 0.0 {
            return Err(StatisticsError::new(
                "harmonic_mean requires positive values",
            ));
        }
        reciprocal_total += 1.0 / value;
    }

    Ok(values.len() as f64 / reciprocal_total)
}

fn sorted_floats(values: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted
}

fn median_low(values: &[f64]) -> Result<f64, StatisticsError> {
    if values.is_empty() {
        return Err(StatisticsError::new(
            "median_low requires at least one data point",
        ));
    }

    let sorted = sorted_floats(values);
    let mid = sorted.len() / 2;
    Ok(if sorted.len() % 2 == 0 {
        sorted[mid - 1]
    } else {
        sorted[mid]
    })
}

fn median_high(values: &[f64]) -> Result<f64, StatisticsError> {
    if values.is_empty() {
        return Err(StatisticsError::new(
            "median_high requires at least one data point",
        ));
    }

    let sorted = sorted_floats(values);
    Ok(sorted[sorted.len() / 2])
}

fn capwords(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                })
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn stem(path: &str) -> String {
    let base = basename(path);
    match base.rsplit_once('.') {
        Some((head, _)) if !head.is_empty() => head.to_string(),
        _ => base.to_string(),
    }
}

fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
        || path.starts_with('\\')
        || matches!(path.as_bytes(), [drive, b':', sep, ..] if drive.is_ascii_alphabetic() && (*sep == b'/' || *sep == b'\\'))
}

fn bisect_left<T>(values: &[T], needle: &T) -> i64
where
    T: PartialOrd,
{
    let mut left = 0_usize;
    let mut right = values.len();

    while left < right {
        let mid = (left + right) / 2;
        let order = values[mid].partial_cmp(needle).unwrap_or(Ordering::Greater);
        if order == Ordering::Less {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left as i64
}

fn pairwise<T: Clone>(values: &[T]) -> Vec<Vec<T>> {
    values
        .windows(2)
        .map(|pair| pair.to_vec())
        .collect::<Vec<_>>()
}

fn batched<T: Clone>(values: &[T], size: i64) -> Result<Vec<Vec<T>>, ValueError> {
    if size <= 0 {
        return Err(ValueError::new("batched: n must be > 0"));
    }

    Ok(values
        .chunks(size as usize)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>())
}

fn main() {
    println!("=== Math Functions ===");
    println!("factorial(10) = {}", factorial(10));
    println!("gcd(48, 18) = {}", gcd(48, 18));
    println!("lcm(4, 6) = {}", lcm(4, 6));
    println!("comb(10, 3) = {}", comb(10, 3));
    println!("perm(5, 3) = {}", perm(5, 3));

    let nums = vec![1_i64, 2, 3, 4, 5];
    println!("prod([1,2,3,4,5]) = {}", prod(&nums));
    println!("exp(1.0) = {}", exp(1.0));
    println!("isfinite(1.0) = {}", isfinite(1.0));

    println!("=== Statistics Functions ===");
    let data = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let even = vec![1.0_f64, 2.0, 3.0, 4.0];
    match (harmonic_mean(&data), median_low(&even), median_high(&even)) {
        (Ok(hm), Ok(ml), Ok(mh)) => {
            println!("harmonic_mean = {hm}");
            println!("median_low = {ml}");
            println!("median_high = {mh}");
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            println!("statistics error: {}", error.message);
        }
    }

    println!("=== String Functions ===");
    println!("capwords = {}", capwords("hello world test"));

    println!("=== Path Functions ===");
    println!("stem = {}", stem("/docs/report.pdf"));
    println!("is_absolute = {}", is_absolute("/usr/bin"));

    println!("=== Generic Bisect ===");
    let floats = vec![1.0_f64, 2.0, 3.0, 4.0, 5.0];
    println!(
        "bisect_left(floats, 2.5) = {}",
        bisect_left(&floats, &2.5_f64)
    );

    println!("=== Itertools ===");
    let items = vec![1_i64, 2, 3, 4];
    println!("pairwise = {:?}", pairwise(&items));

    let items2 = vec![1_i64, 2, 3, 4, 5, 6, 7, 8];
    match batched(&items2, 3) {
        Ok(batches) => println!("batched = {batches:?}"),
        Err(error) => println!("error: {}", error.message),
    }

    println!("Done!");
}
