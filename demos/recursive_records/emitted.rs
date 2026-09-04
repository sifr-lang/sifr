// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entry {
    value: SifrInt,
    next: Option<Box<Self>>,
}
impl Entry {
    const fn new(value: SifrInt, next: Option<Box<Self>>) -> Self {
        let sifr_generated_field_value_7ce4fd9430e80cea_76616c7565: SifrInt = value;
        let sifr_generated_field_value_e5316cbaa025f028_6e657874: Option<Box<Self>> = next;
        Self {
            value: sifr_generated_field_value_7ce4fd9430e80cea_76616c7565,
            next: sifr_generated_field_value_e5316cbaa025f028_6e657874,
        }
    }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn chain_text(entry: Option<&Entry>) -> String {
    let Some(entry) = entry.as_ref() else {
        return ".".to_string();
    };
    let next_entry: Option<&Entry> = entry.next.as_deref();
    {
        let mut sifr_generated_concat: String =
            String::with_capacity(0usize.saturating_add(2usize).saturating_add(0usize));
        sifr_generated_concat.push_str(entry.value.to_string().as_str());
        sifr_generated_concat.push_str("->");
        sifr_generated_concat.push_str(chain_text(next_entry).as_str());
        sifr_generated_concat
    }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn second_value(entry: Option<&Entry>) -> SifrInt {
    let Some(entry) = entry.as_ref() else {
        return SifrInt::from_i64(0);
    };
    let next_entry: Option<&Entry> = entry.next.as_deref();
    let Some(next_entry_value_1788d7cc138c4323) = next_entry else {
        return SifrInt::from_i64(0);
    };
    next_entry_value_1788d7cc138c4323.value
}
fn main() {
    let chain: Option<Entry> = Some(Entry::new(
        SifrInt::from_i64(4),
        Some(Box::new(Entry::new(
            SifrInt::from_i64(2),
            Some(Box::new(Entry::new(SifrInt::from_i64(6), None))),
        ))),
    ));
    let short: Option<Entry> = Some(Entry::new(SifrInt::from_i64(9), None));
    assert_eq!(chain_text(chain.as_ref()), "4->2->6->.");
    assert_eq!(chain_text(None), ".");
    assert_eq!(second_value(chain.as_ref()), SifrInt::from_i64(2));
    assert_eq!(second_value(short.as_ref()), SifrInt::from_i64(0));
    println!("recursive_records: ok");
}
