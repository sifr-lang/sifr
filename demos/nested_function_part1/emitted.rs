fn apply_twice(f: impl Fn(i64) -> i64, value: i64) -> i64 {
    return f(f(value));
}

fn score(base: i64) -> i64 {
    let offset: i64 = 3 as i64;
    let add_offset = |x| {
    return x + offset;
};
    let amplify = |x| {
    return x * (2 as i64);
};
    let adjusted: i64 = apply_twice(add_offset, base);
    return amplify(adjusted);
}

fn bounded_sum(limit: i64) -> i64 {
    fn helper(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            return acc;
        }
        return helper(i + (1 as i64), acc + i, limit);
    }
    return helper(1 as i64, 0 as i64, limit);
}

fn main() {
    assert!(score(4 as i64) == (20 as i64));
    assert!(bounded_sum(5 as i64) == (15 as i64));
}
