// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entry {
    value: i64,
    next: Option<Box<Entry>>,
}

impl Entry {
    fn new(value: i64, next: Option<Box<Entry>>) -> Self {
        let __sifr_field_init_0: i64 = value;
        let __sifr_field_init_1: Option<Box<Entry>> = next;
        Self { value: __sifr_field_init_0, next: __sifr_field_init_1 }
    }
}

impl Entry {
}

fn chain_text(entry: &Option<Entry>) -> String {
    let Some(entry) = entry else {
        return ".".to_string();
    };
    let next_entry: Option<Entry> = (entry.next).as_deref().cloned();
    {
    let mut __sifr_concat: String = String::with_capacity((0usize + 2usize) + 0usize);
    __sifr_concat.push_str((format!("{}", entry.value)).as_str());
    __sifr_concat.push_str("->");
    __sifr_concat.push_str((chain_text(&next_entry)).as_str());
    __sifr_concat
}
}

fn second_value(entry: &Option<Entry>) -> i64 {
    let Some(entry) = entry else {
        return 0_i64;
    };
    let next_entry: Option<Entry> = (entry.next).as_deref().cloned();
    let Some(mut next_entry) = next_entry else {
        return 0_i64;
    };
    next_entry.value
}

fn main() {
    let chain: Option<Entry> = Some(Entry::new(4_i64, Some(Box::new(Entry::new(2_i64, Some(Box::new(Entry::new(6_i64, None))))))));
    let short: Option<Entry> = Some(Entry::new(9_i64, None));
    assert!((chain_text(&chain) == "4->2->6->."));
    assert!((chain_text(&None) == "."));
    assert!((second_value(&chain) == (2_i64)));
    assert!((second_value(&short) == (0_i64)));
    println!("recursive_records: ok");
}
