use std::ops::Add;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ValueError {}

fn chain<'a, T: Clone>(left: &'a [T], right: &'a [T]) -> impl Iterator<Item = T> + 'a {
    left.iter().cloned().chain(right.iter().cloned())
}

fn pairwise<T: Clone>(values: &[T]) -> Vec<(T, T)> {
    values
        .windows(2)
        .map(|window| (window[0].clone(), window[1].clone()))
        .collect()
}

fn batched<T: Clone>(values: &[T], size: usize) -> Result<Vec<Vec<T>>, ValueError> {
    if size == 0 {
        return Err(ValueError::new("batched: n must be > 0"));
    }

    Ok(values.chunks(size).map(|chunk| chunk.to_vec()).collect())
}

fn accumulate<T>(values: &[T]) -> impl Iterator<Item = T> + '_
where
    T: Copy + Add<Output = T>,
{
    values.iter().copied().scan(None, |state, item| {
        let next = state.map_or(item, |sum| sum + item);
        *state = Some(next);
        Some(next)
    })
}

fn cycle_n<'a, T: Clone>(values: &'a [T], n: usize) -> impl Iterator<Item = T> + 'a {
    values.iter().cloned().cycle().take(n)
}

fn collect_core_actual() -> Vec<bool> {
    let batched_ok = batched(&[1, 2, 3, 4, 5], 2)
        .map(|batches| batches == vec![vec![1, 2], vec![3, 4], vec![5]])
        .unwrap_or(false);

    vec![
        chain(&[1, 2], &[3]).collect::<Vec<_>>() == vec![1, 2, 3],
        pairwise(&[1, 2, 3, 4]) == vec![(1, 2), (2, 3), (3, 4)],
        batched_ok,
        accumulate(&[1, 2, 3]).collect::<Vec<_>>() == vec![1, 3, 6],
        cycle_n(&[5, 6], 5).collect::<Vec<_>>() == vec![5, 6, 5, 6, 5],
    ]
}

fn collect_negative_actual() -> Vec<bool> {
    vec![batched(&[1], 0)
        .err()
        .map(|error| !error.message.is_empty())
        .unwrap_or(false)]
}

fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    target.extend_from_slice(values);
}

fn main() {
    let mut actual = Vec::new();
    append_all(&mut actual, &collect_core_actual());
    append_all(&mut actual, &collect_negative_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("itertools itertools parity demo: pass");
}
