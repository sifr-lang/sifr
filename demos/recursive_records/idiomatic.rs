struct Entry {
    value: i64,
    next: Option<Box<Entry>>,
}

impl Entry {
    fn new(value: i64, next: Option<Box<Entry>>) -> Self {
        Self { value, next }
    }
}

fn chain_text(entry: Option<&Entry>) -> String {
    match entry {
        None => ".".to_string(),
        Some(entry) => format!("{}->{}", entry.value, chain_text(entry.next.as_deref())),
    }
}

fn second_value(entry: Option<&Entry>) -> i64 {
    entry
        .and_then(|entry| entry.next.as_deref())
        .map_or(0, |entry| entry.value)
}

fn main() {
    let chain = Some(Box::new(Entry::new(
        4,
        Some(Box::new(Entry::new(2, Some(Box::new(Entry::new(6, None)))))),
    )));
    let short = Some(Box::new(Entry::new(9, None)));

    assert_eq!(chain_text(chain.as_deref()), "4->2->6->.");
    assert_eq!(chain_text(None), ".");
    assert_eq!(second_value(chain.as_deref()), 2);
    assert_eq!(second_value(short.as_deref()), 0);
    println!("recursive_records: ok");
}
