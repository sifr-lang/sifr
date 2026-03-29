use std::collections::{HashMap, HashSet};

fn tuple_count(values: (i64, i64, i64), needle: i64) -> i64 {
    [values.0, values.1, values.2]
        .into_iter()
        .filter(|value| *value == needle)
        .count() as i64
}

fn tuple_index(values: (i64, i64, i64), needle: i64, start: usize) -> Option<usize> {
    [values.0, values.1, values.2]
        .into_iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, value)| (value == needle).then_some(index))
}

fn main() {
    let mut words = vec!["core".to_string()];
    words.extend("xy".chars().map(|ch| ch.to_string()));
    println!("{words:?}");

    let mut mapping = HashMap::from([("base".to_string(), 1_i64)]);
    mapping.insert("extra".to_string(), 2);
    println!("{}", mapping.remove("missing").unwrap_or(7));

    let mut seen = HashSet::from([1_i64]);
    seen.extend([2, 3, 4, 5]);
    seen = seen
        .symmetric_difference(&HashSet::from([3, 9]))
        .copied()
        .collect();
    println!("{}", seen.contains(&9));

    let pair = (4_i64, 5, 4);
    println!("{}", tuple_count(pair, 4));
    if let Some(index) = tuple_index(pair, 4, 1) {
        println!("{index}");
    }

    println!(
        "{:?}",
        "alpha,beta,gamma".splitn(2, ',').collect::<Vec<_>>()
    );
    println!("{}", "aaaa".replacen('a', "b", 2));
}
