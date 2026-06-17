fn chain<T>(
    left: impl IntoIterator<Item = T>,
    right: impl IntoIterator<Item = T>,
) -> impl Iterator<Item = T> {
    left.into_iter().chain(right)
}

fn count(start: i64, step: i64) -> impl Iterator<Item = i64> {
    std::iter::successors(Some(start), move |current| Some(*current + step))
}

fn square(value: i64) -> i64 {
    value * value
}

fn main() {
    let nums = [1_i64, 2, 3, 4];

    let mut it = nums.iter().copied();
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));

    assert_eq!(
        format!("{:?}", nums.iter().copied().map(square).collect::<Vec<_>>()),
        "[1, 4, 9, 16]"
    );
    assert_eq!(
        format!(
            "{:?}",
            nums.iter()
                .copied()
                .filter(|value| value % 2 == 0)
                .collect::<Vec<_>>()
        ),
        "[2, 4]"
    );
    assert_eq!(
        format!(
            "{:?}",
            nums.iter()
                .copied()
                .zip(["a", "b", "c", "d"])
                .map(|(number, text)| (number, text.to_string()))
                .collect::<Vec<_>>()
        ),
        "[(1, \"a\"), (2, \"b\"), (3, \"c\"), (4, \"d\")]"
    );
    assert_eq!(
        format!(
            "{:?}",
            ["x", "y"]
                .into_iter()
                .enumerate()
                .map(|(index, text)| (index as i64 + 10, text.to_string()))
                .collect::<Vec<_>>()
        ),
        "[(10, \"x\"), (11, \"y\")]"
    );
    assert_eq!(
        format!("{:?}", nums.iter().copied().rev().collect::<Vec<_>>()),
        "[4, 3, 2, 1]"
    );
    assert_eq!(
        format!("{:?}", chain([1_i64, 2], [3]).collect::<Vec<_>>()),
        "[1, 2, 3]"
    );

    let mut ticker = count(3, 2);
    assert_eq!(ticker.next(), Some(3));
    assert_eq!(ticker.next(), Some(5));
    assert_eq!(ticker.next(), Some(7));

    println!("iter_fix_lazy_iterators_basics_lock_demo: ok");
}
