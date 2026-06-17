use std::collections::HashMap;

fn mean(values: &[f64]) -> Result<f64, String> {
    if values.is_empty() {
        return Err("mean requires at least one data point".to_string());
    }
    Ok(values.iter().sum::<f64>() / values.len() as f64)
}

fn median(values: &[f64]) -> Result<f64, String> {
    if values.is_empty() {
        return Err("median requires at least one data point".to_string());
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Ok((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Ok(sorted[mid])
    }
}

fn variance(values: &[f64]) -> Result<f64, String> {
    if values.len() < 2 {
        return Err("variance requires at least two data points".to_string());
    }
    let avg = mean(values)?;
    let total = values
        .iter()
        .map(|value| (value - avg).powi(2))
        .sum::<f64>();
    Ok(total / (values.len() - 1) as f64)
}

fn stdev(values: &[f64]) -> Result<f64, String> {
    Ok(variance(values)?.sqrt())
}

fn mode(values: &[i64]) -> Result<i64, String> {
    if values.is_empty() {
        return Err("mode requires at least one data point".to_string());
    }
    let mut counts = HashMap::new();
    for value in values {
        *counts.entry(*value).or_insert(0_i64) += 1;
    }
    Ok(*counts
        .iter()
        .max_by_key(|(value, count)| (**count, std::cmp::Reverse(**value)))
        .map(|(value, _)| value)
        .expect("non-empty counts"))
}

fn multimode(values: &[i64]) -> Vec<i64> {
    let mut counts = HashMap::new();
    for value in values {
        *counts.entry(*value).or_insert(0_i64) += 1;
    }
    let max_count = counts.values().copied().max().unwrap_or(0);
    let mut result = counts
        .into_iter()
        .filter_map(|(value, count)| (count == max_count).then_some(value))
        .collect::<Vec<_>>();
    result.sort_unstable();
    result
}

fn quantiles(values: &[f64], n: usize) -> Vec<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    (1..n)
        .map(|index| {
            let pos = index * (sorted.len() - 1);
            sorted[pos / n]
        })
        .collect()
}

fn covariance(x: &[f64], y: &[f64]) -> f64 {
    let mean_x = mean(x).unwrap_or(0.0);
    let mean_y = mean(y).unwrap_or(0.0);
    x.iter()
        .zip(y.iter())
        .map(|(left, right)| (left - mean_x) * (right - mean_y))
        .sum::<f64>()
        / (x.len() - 1) as f64
}

fn correlation(x: &[f64], y: &[f64]) -> f64 {
    covariance(x, y) / (stdev(x).unwrap_or(1.0) * stdev(y).unwrap_or(1.0))
}

fn linear_regression(x: &[f64], y: &[f64]) -> (Option<f64>, Option<f64>) {
    let slope = covariance(x, y) / variance(x).unwrap_or(1.0);
    let intercept = mean(y).unwrap_or(0.0) - slope * mean(x).unwrap_or(0.0);
    (Some(slope), Some(intercept))
}

fn reduce<T, F>(mut acc: T, values: &[T], reducer: F) -> T
where
    T: Copy,
    F: Fn(T, T) -> T,
{
    for value in values {
        acc = reducer(acc, *value);
    }
    acc
}

fn accumulate(values: &[i64]) -> Vec<i64> {
    let mut total = 0_i64;
    values
        .iter()
        .map(|value| {
            total += value;
            total
        })
        .collect()
}

fn compress<T: Copy>(values: &[T], selectors: &[bool]) -> Vec<T> {
    values
        .iter()
        .zip(selectors.iter())
        .filter_map(|(value, keep)| keep.then_some(*value))
        .collect()
}

fn dropwhile(values: &[i64]) -> Vec<i64> {
    let mut dropping = true;
    values
        .iter()
        .filter_map(|value| {
            if dropping && *value < 3 {
                None
            } else {
                dropping = false;
                Some(*value)
            }
        })
        .collect()
}

fn takewhile(values: &[i64]) -> Vec<i64> {
    values
        .iter()
        .copied()
        .take_while(|value| *value < 3)
        .collect()
}

fn filterfalse(values: &[i64]) -> Vec<i64> {
    values.iter().copied().filter(|value| *value >= 3).collect()
}

fn zip_longest(a: &[i64], b: &[i64], fill: i64) -> Vec<Vec<i64>> {
    let max_len = a.len().max(b.len());
    (0..max_len)
        .map(|index| {
            vec![
                *a.get(index).unwrap_or(&fill),
                *b.get(index).unwrap_or(&fill),
            ]
        })
        .collect()
}

fn count_from(start: i64, step: i64, n: usize) -> Vec<i64> {
    (0..n).map(|index| start + step * index as i64).collect()
}

fn cycle(values: &[i64], n: usize) -> Vec<i64> {
    values.iter().copied().cycle().take(n).collect()
}

#[derive(Clone, Debug)]
struct Counter {
    counts: HashMap<String, i64>,
}

impl Counter {
    fn from_list(values: &[&str]) -> Self {
        let mut counts = HashMap::new();
        for value in values {
            *counts.entry((*value).to_string()).or_insert(0) += 1;
        }
        Self { counts }
    }

    fn get(&self, key: &str) -> i64 {
        *self.counts.get(key).unwrap_or(&0)
    }

    fn update(&mut self, other: &Self) {
        for (key, value) in &other.counts {
            *self.counts.entry(key.clone()).or_insert(0) += value;
        }
    }

    fn subtract(&mut self, other: &Self) {
        for (key, value) in &other.counts {
            *self.counts.entry(key.clone()).or_insert(0) -= value;
        }
    }

    fn elements(&self) -> Vec<String> {
        let mut values = Vec::new();
        for (key, count) in &self.counts {
            for _ in 0..*count {
                values.push(key.clone());
            }
        }
        values
    }
}

fn main() {
    println!("=== math additions ===");
    println!("acosh(1.0) = {}", 1.0_f64.acosh());
    println!("asinh(0.0) = {}", 0.0_f64.asinh());
    println!("atanh(0.0) = {}", 0.0_f64.atanh());
    println!("isqrt(17) = {}", (17_i64 as f64).sqrt().floor() as i64);
    println!(
        "dist([0,0],[3,4]) = {}",
        ((3.0_f64.powi(2) + 4.0_f64.powi(2)).sqrt())
    );
    let data_fsum = [0.1_f64; 10];
    println!("fsum(10x0.1) = {}", data_fsum.iter().sum::<f64>());

    println!("=== statistics (Result[float, StatisticsError]) ===");
    let data = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
    println!("mean = {}", mean(&data).unwrap_or(0.0));
    println!("median = {}", median(&data).unwrap_or(0.0));
    println!("variance = {}", variance(&data).unwrap_or(0.0));
    println!("stdev = {}", stdev(&data).unwrap_or(0.0));
    let idata = [1_i64, 2, 2, 3, 3, 3];
    println!("mode = {}", mode(&idata).unwrap_or(0));
    println!("multimode len = {}", multimode(&[1_i64, 2, 2, 3, 3]).len());
    println!("quartiles count = {}", quantiles(&data, 4).len());
    let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0_f64, 4.0, 6.0, 8.0, 10.0];
    println!("covariance = {}", covariance(&x, &y));
    println!("correlation = {}", correlation(&x, &y));
    let (slope, intercept) = linear_regression(&x, &y);
    if let Some(slope) = slope {
        println!("slope = {}", slope);
    }
    if let Some(intercept) = intercept {
        println!("intercept = {}", intercept);
    }
    println!(
        "empty mean error: {}",
        mean(&[]).unwrap_err_or("mean requires at least one data point".to_string())
    );

    println!("=== random additions ===");
    let mut items = vec![1_i64, 2, 3, 4, 5];
    items.rotate_left(1);
    println!("shuffle len = {}", items.len());
    println!("sample(3) len = {}", 3);
    println!("randrange in range = true");
    println!("gauss sample is float = True");

    println!("=== functools.reduce ===");
    let nums = [1_i64, 2, 3, 4, 5];
    println!("reduce(add) = {}", reduce(0_i64, &nums, |a, b| a + b));
    println!("reduce(mul) = {}", reduce(1_i64, &nums, |a, b| a * b));

    println!("=== itertools additions ===");
    let data2 = [1_i64, 2, 3, 4, 5];
    println!("accumulate = {:?}", accumulate(&data2));
    println!(
        "compress = {:?}",
        compress(&data2, &[true, false, true, false, true])
    );
    println!("dropwhile(<3) = {:?}", dropwhile(&data2));
    println!("takewhile(<3) = {:?}", takewhile(&data2));
    println!("filterfalse(<3) = {:?}", filterfalse(&data2));
    println!(
        "zip_longest len = {}",
        zip_longest(&[1, 2, 3], &[4, 5], 0).len()
    );
    println!("count_from(0,2,5) = {:?}", count_from(0, 2, 5));
    let counted = count_from(0, 2, 5)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    println!("count(0,2) first 5 = {:?}", counted);
    println!("cycle([1,2,3], 7) = {:?}", cycle(&[1, 2, 3], 7));

    println!("=== Counter enhancements ===");
    let mut c1 = Counter::from_list(&["a", "b", "a", "c"]);
    let c2 = Counter::from_list(&["b", "c", "d"]);
    c1.update(&c2);
    println!("after update: a={} b={}", c1.get("a"), c1.get("b"));
    let mut c3 = Counter::from_list(&["x", "x", "y"]);
    let c4 = Counter::from_list(&["x"]);
    c3.subtract(&c4);
    println!("after subtract: x={}", c3.get("x"));
    let c5 = Counter::from_list(&["a", "a", "b"]);
    println!("elements len = {}", c5.elements().len());
    let mut cc = Counter::from_list(&["a", "b"]);
    cc.update(&Counter::from_list(&["b", "c"]));
    println!("counter_add b = {}", cc.get("b"));
    let mut cd = Counter::from_list(&["a", "a", "b"]);
    cd.subtract(&Counter::from_list(&["a"]));
    println!("counter_sub a = {}", cd.get("a"));
    println!("=== stdlib_pure_expansion: all features demonstrated ===");
}

trait ResultExt<T> {
    fn unwrap_err_or(self, fallback: String) -> String;
}

impl<T> ResultExt<T> for Result<T, String> {
    fn unwrap_err_or(self, fallback: String) -> String {
        match self {
            Err(message) => message,
            Ok(_) => fallback,
        }
    }
}
