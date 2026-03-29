fn format_optional(value: Option<i64>) -> String {
    value.map_or_else(|| "None".to_string(), |item| item.to_string())
}

fn main() {
    let nums = [3_i64, 1, 2];

    let mut rev_it = nums.into_iter().rev();
    println!("{}", format_optional(rev_it.next()));
    println!("{:?}", rev_it.collect::<Vec<_>>());

    let indexed_it = ["a", "b"]
        .into_iter()
        .enumerate()
        .map(|(index, value)| (index as i64 + 5, value.to_string()));
    println!("{:?}", indexed_it.collect::<Vec<_>>());

    let zipped_it = [1_i64, 2]
        .into_iter()
        .zip(["x", "y"])
        .zip([true, false])
        .map(|((number, text), flag)| (number, text.to_string(), flag));
    println!("{:?}", zipped_it.collect::<Vec<_>>());
}
