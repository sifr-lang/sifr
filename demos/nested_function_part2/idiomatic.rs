fn power_two(exp: i64) -> i64 {
    fn helper(n: i64) -> i64 {
        if n == 0 {
            1
        } else {
            2 * helper(n - 1)
        }
    }
    helper(exp)
}

fn sum_to(limit: i64) -> i64 {
    fn helper(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            acc
        } else {
            helper(i + 1, acc + i, limit)
        }
    }
    helper(1, 0, limit)
}

fn main() {
    assert_eq!(power_two(10), 1024);
    assert_eq!(sum_to(10), 55);
}
