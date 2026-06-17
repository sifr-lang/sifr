fn lt3(x: &i64) -> bool {
    *x < 3
}

fn add2(a: i64, b: i64) -> i64 {
    a + b
}

fn accumulate(values: &[i64]) -> impl Iterator<Item = i64> + '_ {
    values.iter().scan(0, |state, value| {
        *state += value;
        Some(*state)
    })
}

fn compress(values: &[i64], selectors: &[bool]) -> Vec<i64> {
    values
        .iter()
        .zip(selectors)
        .filter_map(|(value, keep)| keep.then_some(*value))
        .collect()
}

fn dropwhile(values: &[i64], predicate: fn(&i64) -> bool) -> Vec<i64> {
    let mut started = false;
    values
        .iter()
        .filter_map(|value| {
            if started || !predicate(value) {
                started = true;
                Some(*value)
            } else {
                None
            }
        })
        .collect()
}

fn takewhile(values: &[i64], predicate: fn(&i64) -> bool) -> Vec<i64> {
    values.iter().copied().take_while(predicate).collect()
}

fn filterfalse(values: &[i64], predicate: fn(&i64) -> bool) -> Vec<i64> {
    values
        .iter()
        .filter(|value| !predicate(value))
        .copied()
        .collect()
}

fn zip_longest(left: &[i64], right: &[i64], fill: i64) -> Vec<Vec<i64>> {
    let max_len = left.len().max(right.len());
    (0..max_len)
        .map(|index| {
            vec![
                left.get(index).copied().unwrap_or(fill),
                right.get(index).copied().unwrap_or(fill),
            ]
        })
        .collect()
}

fn cycle(values: &[i64], count: usize) -> impl Iterator<Item = i64> + '_ {
    values.iter().copied().cycle().take(count)
}

fn starmap(values: &[(i64, i64)], func: fn(i64, i64) -> i64) -> Vec<i64> {
    values.iter().map(|(a, b)| func(*a, *b)).collect()
}

fn product(values: &[i64], repeat: i64) -> Vec<Vec<i64>> {
    if repeat < 0 {
        return vec![];
    }
    if repeat == 0 {
        return vec![vec![]];
    }

    let mut result = vec![Vec::new()];
    for _ in 0..repeat {
        result = result
            .into_iter()
            .flat_map(|prefix| {
                values.iter().map(move |value| {
                    let mut next = prefix.clone();
                    next.push(*value);
                    next
                })
            })
            .collect();
    }
    result
}

fn permutations(values: &[i64], size: usize) -> Vec<Vec<i64>> {
    fn build(prefix: Vec<i64>, rest: Vec<i64>, size: usize, out: &mut Vec<Vec<i64>>) {
        if prefix.len() == size {
            out.push(prefix);
            return;
        }
        for index in 0..rest.len() {
            let mut next_prefix = prefix.clone();
            next_prefix.push(rest[index]);
            let mut next_rest = rest.clone();
            next_rest.remove(index);
            build(next_prefix, next_rest, size, out);
        }
    }

    let mut out = Vec::new();
    build(Vec::new(), values.to_vec(), size, &mut out);
    out
}

fn combinations(values: &[i64], size: usize) -> Vec<Vec<i64>> {
    fn build(
        values: &[i64],
        start: usize,
        size: usize,
        prefix: &mut Vec<i64>,
        out: &mut Vec<Vec<i64>>,
    ) {
        if prefix.len() == size {
            out.push(prefix.clone());
            return;
        }
        for index in start..values.len() {
            prefix.push(values[index]);
            build(values, index + 1, size, prefix, out);
            prefix.pop();
        }
    }

    let mut out = Vec::new();
    build(values, 0, size, &mut Vec::new(), &mut out);
    out
}

fn combinations_with_replacement(values: &[i64], size: usize) -> Vec<Vec<i64>> {
    fn build(
        values: &[i64],
        start: usize,
        size: usize,
        prefix: &mut Vec<i64>,
        out: &mut Vec<Vec<i64>>,
    ) {
        if prefix.len() == size {
            out.push(prefix.clone());
            return;
        }
        for index in start..values.len() {
            prefix.push(values[index]);
            build(values, index, size, prefix, out);
            prefix.pop();
        }
    }

    let mut out = Vec::new();
    build(values, 0, size, &mut Vec::new(), &mut out);
    out
}

fn main() {
    let nums = [1, 2, 3, 4];
    let mut acc_it = accumulate(&nums);
    assert_eq!(acc_it.next(), Some(1));
    assert_eq!(format!("{:?}", acc_it.collect::<Vec<_>>()), "[3, 6, 10]");

    assert_eq!(
        format!("{:?}", compress(&nums, &[true, false, true, false])),
        "[1, 3]"
    );
    assert_eq!(format!("{:?}", dropwhile(&[1, 2, 3, 1], lt3)), "[3, 1]");
    assert_eq!(format!("{:?}", takewhile(&[1, 2, 3, 1], lt3)), "[1, 2]");
    assert_eq!(format!("{:?}", filterfalse(&[1, 2, 3, 1], lt3)), "[3]");
    assert_eq!(
        format!("{:?}", zip_longest(&[1, 2], &[9], 0)),
        "[[1, 9], [2, 0]]"
    );

    let mut cyc = cycle(&[1, 2, 3], 5);
    assert_eq!(cyc.next(), Some(1));
    assert_eq!(format!("{:?}", cyc.collect::<Vec<_>>()), "[2, 3, 1, 2]");

    assert_eq!(format!("{:?}", starmap(&[(2, 3), (4, 5)], add2)), "[5, 9]");
    assert_eq!(
        format!("{:?}", product(&[1, 2], 2)),
        "[[1, 1], [1, 2], [2, 1], [2, 2]]"
    );
    assert_eq!(format!("{:?}", product(&[1, 2], -1)), "[]");
    assert_eq!(
        format!("{:?}", permutations(&[1, 2, 3], 2)),
        "[[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]]"
    );
    assert_eq!(
        format!("{:?}", combinations(&[1, 2, 3], 2)),
        "[[1, 2], [1, 3], [2, 3]]"
    );
    assert_eq!(
        format!("{:?}", combinations_with_replacement(&[1, 2], 2)),
        "[[1, 1], [1, 2], [2, 2]]"
    );

    println!("parity_ext_extended_itertools_lazy_surface_demo: ok");
}
