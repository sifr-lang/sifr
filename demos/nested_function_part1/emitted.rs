// src/main.rs
fn apply_twice(f: impl Fn(i64) -> i64, value: i64) -> i64 {
    f(f(value))
}

fn score(base: i64) -> i64 {
    let offset: i64 = 3_i64;
    let add_offset = |x: i64| {
    x + offset
};
    let amplify = |x: i64| {
    x * (2_i64)
};
    let adjusted: i64 = apply_twice(add_offset, base);
    amplify(adjusted)
}

fn bounded_sum(limit: i64) -> i64 {
    fn helper(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            return acc;
        }
        return helper(i + (1_i64), acc + i, limit);
    }
    helper(1_i64, 0_i64, limit)
}

fn main() {
    assert!((score(4_i64) == (20_i64)));
    assert!((bounded_sum(5_i64) == (15_i64)));
}
