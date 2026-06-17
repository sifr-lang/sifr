use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone)]
struct StatisticsError {
    message: String,
}

impl StatisticsError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for StatisticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for StatisticsError {}

type StatisticsResult<T> = Result<T, StatisticsError>;

fn near(value: f64, target: f64, tolerance: f64) -> bool {
    (value - target).abs() <= tolerance
}

fn mean(data: &[f64]) -> StatisticsResult<f64> {
    if data.is_empty() {
        return Err(StatisticsError::new(
            "mean requires at least one data point",
        ));
    }

    Ok(data.iter().sum::<f64>() / data.len() as f64)
}

fn median(data: &[f64]) -> StatisticsResult<f64> {
    let mut sorted = data.to_vec();
    if sorted.is_empty() {
        return Err(StatisticsError::new(
            "median requires at least one data point",
        ));
    }

    sorted.sort_by(f64::total_cmp);
    let midpoint = sorted.len() / 2;

    if sorted.len() % 2 == 0 {
        Ok((sorted[midpoint - 1] + sorted[midpoint]) / 2.0)
    } else {
        Ok(sorted[midpoint])
    }
}

fn variance(data: &[f64]) -> StatisticsResult<f64> {
    if data.len() < 2 {
        return Err(StatisticsError::new(
            "variance requires at least two data points",
        ));
    }

    let average = mean(data)?;
    let squared_diffs = data
        .iter()
        .map(|value| {
            let delta = value - average;
            delta * delta
        })
        .sum::<f64>();

    Ok(squared_diffs / (data.len() - 1) as f64)
}

fn stdev(data: &[f64]) -> StatisticsResult<f64> {
    Ok(variance(data)?.sqrt())
}

fn mode(data: &[i64]) -> StatisticsResult<i64> {
    if data.is_empty() {
        return Err(StatisticsError::new(
            "mode requires at least one data point",
        ));
    }

    let mut counts = HashMap::new();
    for value in data {
        *counts.entry(*value).or_insert(0_usize) += 1;
    }

    let mut best_value = data[0];
    let mut best_count = 0_usize;
    for value in data {
        let count = counts[value];
        if count > best_count {
            best_count = count;
            best_value = *value;
        }
    }

    Ok(best_value)
}

fn multimode(data: &[i64]) -> StatisticsResult<Vec<i64>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut counts = HashMap::new();
    for value in data {
        *counts.entry(*value).or_insert(0_usize) += 1;
    }

    let Some(max_count) = counts.values().copied().max() else {
        return Ok(Vec::new());
    };

    let mut seen = HashSet::new();
    let mut modes = Vec::new();
    for value in data {
        if counts[value] == max_count && seen.insert(*value) {
            modes.push(*value);
        }
    }

    Ok(modes)
}

fn quantiles(data: &[f64], n: usize) -> StatisticsResult<Vec<f64>> {
    if data.len() < 2 {
        return Err(StatisticsError::new(
            "quantiles requires at least two data points",
        ));
    }
    if n < 1 {
        return Err(StatisticsError::new("quantiles requires n >= 1"));
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(f64::total_cmp);

    let mut result = Vec::with_capacity(n.saturating_sub(1));
    for index in 1..n {
        let position = (sorted.len() - 1) as f64 * index as f64 / n as f64;
        let lower = position.floor() as usize;
        let upper = position.ceil() as usize;

        if lower == upper {
            result.push(sorted[lower]);
        } else {
            let fraction = position - lower as f64;
            result.push(sorted[lower] + (sorted[upper] - sorted[lower]) * fraction);
        }
    }

    Ok(result)
}

fn covariance(x: &[f64], y: &[f64]) -> StatisticsResult<f64> {
    if x.len() != y.len() {
        return Err(StatisticsError::new(
            "covariance: x and y must have the same length",
        ));
    }
    if x.len() < 2 {
        return Err(StatisticsError::new(
            "covariance requires at least two paired data points",
        ));
    }

    let mean_x = mean(x)?;
    let mean_y = mean(y)?;
    let numerator = x
        .iter()
        .zip(y)
        .map(|(left, right)| (left - mean_x) * (right - mean_y))
        .sum::<f64>();

    Ok(numerator / (x.len() - 1) as f64)
}

fn correlation(x: &[f64], y: &[f64]) -> StatisticsResult<f64> {
    let deviation_x = stdev(x)?;
    let deviation_y = stdev(y)?;

    if deviation_x == 0.0 {
        return Err(StatisticsError::new("correlation: x has zero variance"));
    }
    if deviation_y == 0.0 {
        return Err(StatisticsError::new("correlation: y has zero variance"));
    }

    Ok(covariance(x, y)? / (deviation_x * deviation_y))
}

fn linear_regression(x: &[f64], y: &[f64]) -> StatisticsResult<(f64, f64)> {
    let slope = covariance(x, y)? / variance(x)?;
    let intercept = mean(y)? - slope * mean(x)?;
    Ok((slope, intercept))
}

fn harmonic_mean(data: &[f64]) -> StatisticsResult<f64> {
    if data.is_empty() {
        return Err(StatisticsError::new(
            "harmonic_mean requires at least one data point",
        ));
    }
    if data.iter().any(|value| *value <= 0.0) {
        return Err(StatisticsError::new(
            "harmonic_mean requires positive values",
        ));
    }

    let reciprocal_sum = data.iter().map(|value| 1.0 / value).sum::<f64>();
    Ok(data.len() as f64 / reciprocal_sum)
}

fn geometric_mean(data: &[f64]) -> StatisticsResult<f64> {
    if data.is_empty() {
        return Err(StatisticsError::new(
            "geometric_mean requires at least one data point",
        ));
    }
    if data.iter().any(|value| *value <= 0.0) {
        return Err(StatisticsError::new(
            "geometric_mean requires positive values",
        ));
    }

    let log_sum = data.iter().map(|value| value.ln()).sum::<f64>();
    Ok((log_sum / data.len() as f64).exp())
}

fn collect_positive_actual() -> Vec<String> {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0];
    let paired_y = [2.0, 4.0, 6.0, 8.0, 10.0];

    let (slope_ok, intercept_ok) = linear_regression(&data, &paired_y)
        .map(|(slope, intercept)| (near(slope, 2.0, 0.0001), near(intercept, 0.0, 0.0001)))
        .unwrap_or((false, false));

    vec![
        mean(&data)
            .map(|value| near(value, 3.0, 0.0001))
            .unwrap_or(false)
            .to_string(),
        median(&data)
            .map(|value| near(value, 3.0, 0.0001))
            .unwrap_or(false)
            .to_string(),
        variance(&data)
            .map(|value| near(value, 2.5, 0.0001))
            .unwrap_or(false)
            .to_string(),
        stdev(&data)
            .map(|value| near(value, 1.5811, 0.001))
            .unwrap_or(false)
            .to_string(),
        mode(&[1, 2, 2, 3, 3, 3])
            .map(|value| value == 3)
            .unwrap_or(false)
            .to_string(),
        multimode(&[1, 2, 2, 3, 3])
            .map(|values| values.len() == 2)
            .unwrap_or(false)
            .to_string(),
        quantiles(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], 4)
            .map(|values| values.len() == 3)
            .unwrap_or(false)
            .to_string(),
        covariance(&data, &paired_y)
            .map(|value| near(value, 5.0, 0.0001))
            .unwrap_or(false)
            .to_string(),
        correlation(&data, &paired_y)
            .map(|value| near(value, 1.0, 0.0001))
            .unwrap_or(false)
            .to_string(),
        (slope_ok && intercept_ok).to_string(),
        harmonic_mean(&[2.0, 4.0, 4.0, 8.0])
            .map(|value| near(value, 3.5555555556, 0.0001))
            .unwrap_or(false)
            .to_string(),
        geometric_mean(&[4.0, 9.0])
            .map(|value| near(value, 6.0, 0.0001))
            .unwrap_or(false)
            .to_string(),
    ]
}

fn collect_error_actual_ok() -> Vec<bool> {
    vec![mean(&[]).is_ok(), harmonic_mean(&[0.0, 1.0]).is_ok()]
}

fn main() {
    let expected = vec![
        "true", "true", "true", "true", "true", "true", "true", "true", "true", "true", "true",
        "true",
    ];
    let actual = collect_positive_actual();
    assert_eq!(actual, expected);

    let expected_ok = vec![false, false];
    let actual_ok = collect_error_actual_ok();
    assert_eq!(actual_ok, expected_ok);

    println!("statistics parity demo: pass");
}
