fn power_two(exp: i64) -> i64 {
    fn helper(n: i64) -> i64 {
        if n == (0 as i64) {
            return 1 as i64;
        }
        return (2 as i64) * helper(n - (1 as i64));
    }
    return helper(exp);
}

fn sum_to(limit: i64) -> i64 {
    fn helper(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            return acc;
        }
        return helper(i + (1 as i64), acc + i, limit);
    }
    return helper(1 as i64, 0 as i64, limit);
}

fn main() {
    assert!(power_two(10 as i64) == (1024 as i64));
    assert!(sum_to(10 as i64) == (55 as i64));
}
