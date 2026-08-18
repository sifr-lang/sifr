// src/main.rs
fn power_two(exp: i64) -> i64 {
    fn helper(n: i64) -> i64 {
        if n == (0_i64) {
            return 1_i64;
        }
        return (2_i64) * helper(n - (1_i64));
    }
    helper(exp)
}

fn sum_to(limit: i64) -> i64 {
    fn helper(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            return acc;
        }
        return helper(i + (1_i64), acc + i, limit);
    }
    helper(1_i64, 0_i64, limit)
}

fn main() {
    assert!((power_two(10_i64) == (1024_i64)));
    assert!((sum_to(10_i64) == (55_i64)));
}
