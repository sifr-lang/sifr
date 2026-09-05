// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entry {
    value: SifrInt,
    next: Option<Box<Self>>,
}
impl Entry {
    fn new(value: &SifrInt, next: Option<Box<Self>>) -> Self {
        let sifr_generated_field_value_7ce4fd9430e80cea_76616c7565: SifrInt = (*value).clone();
        let sifr_generated_field_value_e5316cbaa025f028_6e657874: Option<Box<Self>> = next;
        Self {
            value: sifr_generated_field_value_7ce4fd9430e80cea_76616c7565,
            next: sifr_generated_field_value_e5316cbaa025f028_6e657874,
        }
    }
}
fn collect_values(root: Option<&Entry>) -> String {
    fn visit(node: Option<&Entry>, values: &mut Vec<SifrInt>) {
        let Some(node) = node.as_ref() else {
            return;
        };
        values.push(node.value.clone());
        visit(node.next.as_deref(), values);
    }
    let mut values: Vec<SifrInt> = Vec::new();
    visit(root, &mut values);
    format!("{values:?}")
}
fn main() {
    let short: Entry = Entry::new(
        &SifrInt::from_i64(1),
        Some(Box::new(Entry::new(&SifrInt::from_i64(2), None))),
    );
    let long: Entry = Entry::new(
        &SifrInt::from_i64(4),
        Some(Box::new(Entry::new(
            &SifrInt::from_i64(5),
            Some(Box::new(Entry::new(&SifrInt::from_i64(6), None))),
        ))),
    );
    assert_eq!(collect_values(None), "[]");
    assert_eq!(collect_values(Some(&short)), "[1, 2]");
    assert_eq!(collect_values(Some(&long)), "[4, 5, 6]");
    println!("nested_recursive_helpers: ok");
}
