// src/main.rs
use ::sifr_runtime::SifrInt;
const INF: f64 = f64::INFINITY;
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn cbrt(x: f64) -> f64 {
    ::sifr_stdlib::math::cbrt(x)
}
fn exp2(x: f64) -> f64 {
    ::sifr_stdlib::math::exp2(x)
}
const fn isnan(x: f64) -> bool {
    ::sifr_stdlib::math::isnan(x)
}
const fn isinf(x: f64) -> bool {
    ::sifr_stdlib::math::isinf(x)
}
fn remainder(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::remainder(x, y)
}
fn fma(x: f64, y: f64, z: f64) -> f64 {
    ::sifr_stdlib::math::fma(x, y, z)
}
const fn isnormal(x: f64) -> bool {
    ::sifr_stdlib::math::isnormal(x)
}
fn issubnormal(x: f64) -> bool {
    ::sifr_stdlib::math::issubnormal(x)
}
fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::dist(p, q)
}
fn fsum_impl(data: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::fsum(data)
}
fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::sumprod(p, q)
}
fn nextafter(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::nextafter(x, y)
}
fn ulp(x: f64) -> f64 {
    ::sifr_stdlib::math::ulp(x)
}
fn log_base(x: f64, base: f64) -> f64 {
    log(x) / log(base)
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < 0.0_f64 {
        return false;
    }
    if abs_tol < 0.0_f64 {
        return false;
    }
    if a == b {
        return true;
    }
    if isnan(a) || isnan(b) {
        return false;
    }
    if isinf(a) || isinf(b) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < 0.0_f64 {
        diff = 0.0_f64 - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < 0.0_f64 {
        a_abs = 0.0_f64 - a_abs;
    }
    let mut b_abs_value_a5463241d121f11a: f64 = b;
    if b_abs_value_a5463241d121f11a < 0.0_f64 {
        b_abs_value_a5463241d121f11a = 0.0_f64 - b_abs_value_a5463241d121f11a;
    }
    let mut larger_abs: f64 = a_abs;
    if b_abs_value_a5463241d121f11a > larger_abs {
        larger_abs = b_abs_value_a5463241d121f11a;
    }
    let mut rel_bound: f64 = rel_tol * larger_abs;
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    diff <= rel_bound
}
fn sifr_generated_copy_float_list(data: &[f64]) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &[f64], q: &[f64]) -> f64 {
    dist_impl(
        sifr_generated_copy_float_list(p),
        sifr_generated_copy_float_list(q),
    )
}
fn fsum(data: &[f64]) -> f64 {
    fsum_impl(sifr_generated_copy_float_list(data))
}
fn sumprod(p: &[f64], q: &[f64]) -> f64 {
    sumprod_impl(
        sifr_generated_copy_float_list(p),
        sifr_generated_copy_float_list(q),
    )
}
fn assert_vector_eq(actual: &[String], expected: &[String]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert_eq!(
            {
                let sifr_generated_condition_list = &actual;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .cloned()
            },
            {
                let sifr_generated_condition_list = &expected;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .cloned()
            }
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert_eq!(
            {
                let sifr_generated_condition_list = &actual;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .copied()
            },
            {
                let sifr_generated_condition_list = &expected;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .copied()
            }
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = vec![
        (cbrt(27.0_f64) == 3.0_f64).to_string(),
        (exp2(3.0_f64) == 8.0_f64).to_string(),
        (fma(2.0_f64, 3.0_f64, 4.0_f64) == 10.0_f64).to_string(),
        (log_base(32.0_f64, 2.0_f64) == 5.0_f64).to_string(),
        isclose(0.000_000_001_f64, 0.0_f64, 0.9_f64, 0.000_000_01_f64).to_string(),
    ];
    let p: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64];
    let q: Vec<f64> = vec![4.0_f64, 5.0_f64, 6.0_f64];
    actual.push((sumprod(&p, &q) == 32.0_f64).to_string());
    let tiny_subnormal: f64 = 0.000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_022_250_738_585_072_014_f64
        / 2.0_f64;
    actual.push(isnormal(1.0_f64).to_string());
    actual.push(issubnormal(tiny_subnormal).to_string());
    actual.push((remainder(5.5_f64, 2.0_f64) < 0.0_f64).to_string());
    actual.push(isnan(dist(&vec![1.0_f64, 2.0_f64], &vec![1.0_f64])).to_string());
    actual
        .push(
            (fsum(
                &vec![
                    10_000_000_000_000_000_159_028_911_097_599_180_468_360_808_563_945_281_389_781_327_557_747_838_772_170_381_060_813_469_985_856_815_104.0_f64,
                    1.0_f64, -
                    10_000_000_000_000_000_159_028_911_097_599_180_468_360_808_563_945_281_389_781_327_557_747_838_772_170_381_060_813_469_985_856_815_104.0_f64
                ],
            ) == 1.0_f64)
                .to_string(),
        );
    actual.push((nextafter(1.0_f64, INF) > 1.0_f64).to_string());
    actual.push((ulp(1.0_f64) > 0.0_f64).to_string());
    actual
}
fn collect_negative_actual_false() -> Vec<bool> {
    vec![
        isclose(1.0_f64, 1.0_f64, -0.1_f64, 0.0_f64),
        isclose(1.0_f64, 1.0_f64, 0.1_f64, -0.1_f64),
    ]
}
fn main() {
    let expected: Vec<String> = vec![
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
    let actual: Vec<String> = collect_positive_actual();
    assert_vector_eq(&actual, &expected);
    let expected_false: Vec<bool> = vec![false, false];
    let actual_false: Vec<bool> = collect_negative_actual_false();
    assert_bool_vector_eq(&actual_false, &expected_false);
    println!("math math parity demo: pass");
}
