fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn bisect_left<T: Ord>(values: &[T], needle: &T) -> usize {
    values.partition_point(|value| value < needle)
}

fn bisect_right<T: Ord>(values: &[T], needle: &T) -> usize {
    values.partition_point(|value| value <= needle)
}

fn insort_left<T: Ord>(values: &mut Vec<T>, needle: T) {
    let index = bisect_left(values, &needle);
    values.insert(index, needle);
}

fn insort_right<T: Ord>(values: &mut Vec<T>, needle: T) {
    let index = bisect_right(values, &needle);
    values.insert(index, needle);
}

fn collect_actual() -> Vec<bool> {
    let data = vec![1, 2, 2, 3, 5];

    let mut left_mut = vec![1, 3, 3, 5];
    insort_left(&mut left_mut, 3);

    let mut right_mut = vec![1, 3, 3, 5];
    insort_right(&mut right_mut, 3);

    let mut empty = Vec::new();
    let empty_left = bisect_left(&empty, &10);
    insort_right(&mut empty, 10);

    vec![
        bisect_left(&data, &2) == 1,
        bisect_right(&data, &2) == 3,
        bisect_left(&data, &4) == 4,
        bisect_right(&data, &4) == 4,
        left_mut == vec![1, 3, 3, 3, 5],
        right_mut == vec![1, 3, 3, 3, 5],
        empty_left == 0,
        empty == vec![10],
    ]
}

fn main() {
    let actual = collect_actual();
    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true, true, true]);
    println!("bisect bisect parity demo: pass");
}
