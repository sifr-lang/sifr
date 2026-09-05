fn apply_twice(f: impl Fn(i64) -> i64, value: i64) -> i64 {
    f(f(value))
}

fn score(base: i64) -> i64 {
    let offset = 3;
    let add_offset = |x| x + offset;
    let amplify = |x| x * 2;
    let adjusted = apply_twice(add_offset, base);
    amplify(adjusted)
}

fn bounded_sum(limit: i64) -> i64 {
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
    assert_eq!(score(4), 20);
    assert_eq!(bounded_sum(5), 15);
}
