fn add(x: i64, y: i64) -> i64 {
    x + y
}

fn main() {
    let mut rev_it = [9_i64, 7, 5].into_iter().rev();
    assert_eq!(rev_it.next(), Some(5));
    assert_eq!(format!("{:?}", rev_it.collect::<Vec<_>>()), "[7, 9]");

    let enum_it = ["a", "b"]
        .into_iter()
        .enumerate()
        .map(|(index, value)| (index as i64 + 3, value.to_string()));
    assert_eq!(
        format!("{:?}", enum_it.collect::<Vec<_>>()),
        "[(3, \"a\"), (4, \"b\")]"
    );

    let zip_it = [1_i64, 2]
        .into_iter()
        .zip(["x", "y"])
        .map(|(number, text)| (number, text.to_string()));
    assert_eq!(
        format!("{:?}", zip_it.collect::<Vec<_>>()),
        "[(1, \"x\"), (2, \"y\")]"
    );

    let mut mapped_it = [1_i64, 2, 3]
        .into_iter()
        .zip([4_i64, 5, 6])
        .map(|(x, y)| add(x, y));
    assert_eq!(mapped_it.next(), Some(5));
    assert_eq!(format!("{:?}", mapped_it.collect::<Vec<_>>()), "[7, 9]");

    assert_eq!(
        "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok",
        "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok"
    );
}
