// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq)]
struct Payload {
    values: Vec<SifrInt>,
    counts: HashMap<String, SifrInt>,
}
impl Payload {
    const fn new(values: Vec<SifrInt>, counts: HashMap<String, SifrInt>) -> Self {
        Self { values, counts }
    }
}
fn append_default(mut items: Vec<SifrInt>) -> Vec<SifrInt> {
    items.push(SifrInt::from_i64(9));
    items
}
fn main() {
    let first: Vec<SifrInt> = append_default(vec![SifrInt::from_i64(1)]);
    let second: Vec<SifrInt> = append_default(vec![SifrInt::from_i64(1)]);
    let payload: Payload = Payload::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2)], {
        let mut sifr_generated_registry_dict_literal = ::std::collections::HashMap::new();
        sifr_generated_registry_dict_literal.insert("ok".to_string(), SifrInt::from_i64(1));
        sifr_generated_registry_dict_literal
    });
    println!("default_values defaults and panic-to-diagnostic conversion demo:");
    println!("{first:?}");
    println!("{second:?}");
    println!("{:?}", payload.values);
}
