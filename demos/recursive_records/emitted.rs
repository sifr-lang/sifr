// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entry {
    value: SifrInt,
    next: Option<Box<Entry>>,
}

impl Entry {
    fn new(value: SifrInt, next: Option<Box<Entry>>) -> Self {
        let __sifr_field_init_0: SifrInt = value.clone();
        let __sifr_field_init_1: Option<Box<Entry>> = next;
        Self { value: __sifr_field_init_0, next: __sifr_field_init_1 }
    }
}

impl Entry {
}

fn chain_text(entry: &Option<Entry>) -> String {
    let Some(entry) = entry.as_ref() else {
        return ".".to_string();
    };
    let next_entry: Option<Entry> = (entry.next).as_deref().cloned();
    {
    let mut __sifr_concat: String = String::with_capacity((0usize + 2usize) + 0usize);
    __sifr_concat.push_str((format!("{}", entry.value.clone())).as_str());
    __sifr_concat.push_str("->");
    __sifr_concat.push_str((chain_text(&next_entry)).as_str());
    __sifr_concat
}
}

fn second_value(entry: &Option<Entry>) -> SifrInt {
    let Some(entry) = entry.as_ref() else {
        return SifrInt::from_i64(0);
    };
    let next_entry: Option<Entry> = (entry.next).as_deref().cloned();
    let Some(mut next_entry) = next_entry else {
        return SifrInt::from_i64(0);
    };
    next_entry.value.clone()
}

fn main() {
    let chain: Option<Entry> = Some(Entry::new(SifrInt::from_i64(4), Some(Box::new(Entry::new(SifrInt::from_i64(2), Some(Box::new(Entry::new(SifrInt::from_i64(6), None))))))));
    let short: Option<Entry> = Some(Entry::new(SifrInt::from_i64(9), None));
    assert!((chain_text(&chain) == "4->2->6->."));
    assert!((chain_text(&None) == "."));
    assert!((&second_value(&chain) == &SifrInt::from_i64(2)));
    assert!((&second_value(&short) == &SifrInt::from_i64(0)));
    println!("recursive_records: ok");
}
