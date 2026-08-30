// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let values: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    let mut it: Box<dyn Iterator<Item = SifrInt>> = Box::new((values).iter().cloned());
    let first: Option<SifrInt> = it.next();
    println!("{}", (first).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let mut remaining_total: SifrInt = SifrInt::from_i64(0);
    for item in it {
        remaining_total = &remaining_total + &item;
    }
    println!("{}", remaining_total);
    let mut pair_total: SifrInt = SifrInt::from_i64(0);
    for (i, value) in Box::new((values).iter().cloned().enumerate().map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(0), __pair.1))) {
        pair_total = &(&pair_total + &i) + &value;
    }
    println!("{}", pair_total);
}
