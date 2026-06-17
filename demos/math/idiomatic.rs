fn assert_vector_eq(actual: &[String], expected: &[String]) {
    assert_eq!(actual, expected);
}

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn log_base(x: f64, base: f64) -> f64 {
    x.ln() / base.ln()
}

fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < 0.0 || abs_tol < 0.0 {
        return false;
    }
    if a == b {
        return true;
    }
    if a.is_nan() || b.is_nan() || a.is_infinite() || b.is_infinite() {
        return false;
    }

    let diff = (a - b).abs();
    diff <= rel_tol * a.abs().max(b.abs()) || diff <= abs_tol
}

fn sumprod(lhs: &[f64], rhs: &[f64]) -> f64 {
    lhs.iter().zip(rhs).map(|(x, y)| x * y).sum()
}

fn isnormal(value: f64) -> bool {
    value.is_normal()
}

fn issubnormal(value: f64) -> bool {
    value != 0.0 && value.is_finite() && !value.is_normal()
}

fn round_ties_even(value: f64) -> f64 {
    let truncated = value.trunc();
    let fraction = value - truncated;

    if fraction.abs() < 0.5 {
        truncated
    } else if fraction.abs() > 0.5 {
        truncated + value.signum()
    } else if truncated.rem_euclid(2.0) == 0.0 {
        truncated
    } else {
        truncated + value.signum()
    }
}

fn remainder(x: f64, y: f64) -> f64 {
    if !x.is_finite() || !y.is_finite() || y == 0.0 {
        return f64::NAN;
    }
    x - round_ties_even(x / y) * y
}

fn dist(lhs: &[f64], rhs: &[f64]) -> f64 {
    if lhs.len() != rhs.len() {
        return f64::NAN;
    }

    lhs.iter()
        .zip(rhs)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn fsum(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut compensation = 0.0;

    for &value in values {
        let tentative = sum + value;
        if sum.abs() >= value.abs() {
            compensation += (sum - tentative) + value;
        } else {
            compensation += (value - tentative) + sum;
        }
        sum = tentative;
    }

    sum + compensation
}

fn nextafter(from: f64, to: f64) -> f64 {
    if from.is_nan() || to.is_nan() {
        return f64::NAN;
    }
    if from == to {
        return to;
    }
    if from == 0.0 {
        return if to.is_sign_positive() {
            f64::from_bits(1)
        } else {
            -f64::from_bits(1)
        };
    }

    let mut bits = from.to_bits();
    if (to > from) == from.is_sign_positive() {
        bits += 1;
    } else {
        bits -= 1;
    }
    f64::from_bits(bits)
}

fn ulp(value: f64) -> f64 {
    if value.is_nan() {
        f64::NAN
    } else if value.is_infinite() {
        f64::INFINITY
    } else {
        (nextafter(value, f64::INFINITY) - value).abs()
    }
}

fn collect_positive_actual() -> Vec<String> {
    let tiny_subnormal = 2.225_073_858_507_201_4e-308 / 2.0;
    let p = [1.0, 2.0, 3.0];
    let q = [4.0, 5.0, 6.0];

    vec![
        (27.0f64.cbrt() == 3.0).to_string(),
        (3.0f64.exp2() == 8.0).to_string(),
        (2.0f64.mul_add(3.0, 4.0) == 10.0).to_string(),
        (log_base(32.0, 2.0) == 5.0).to_string(),
        isclose(1e-9, 0.0, 0.9, 1e-8).to_string(),
        (sumprod(&p, &q) == 32.0).to_string(),
        isnormal(1.0).to_string(),
        issubnormal(tiny_subnormal).to_string(),
        (remainder(5.5, 2.0) < 0.0).to_string(),
        dist(&[1.0, 2.0], &[1.0]).is_nan().to_string(),
        (fsum(&[1e100, 1.0, -1e100]) == 1.0).to_string(),
        (nextafter(1.0, f64::INFINITY) > 1.0).to_string(),
        (ulp(1.0) > 0.0).to_string(),
    ]
}

fn collect_negative_actual_false() -> Vec<bool> {
    vec![isclose(1.0, 1.0, -0.1, 0.0), isclose(1.0, 1.0, 0.1, -0.1)]
}

fn main() {
    let expected = vec![
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
    ];
    let expected_false = vec![false, false];

    assert_vector_eq(&collect_positive_actual(), &expected);
    assert_bool_vector_eq(&collect_negative_actual_false(), &expected_false);

    println!("math math parity demo: pass");
}
