// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn get_length(items: &Vec<SifrInt>) -> SifrInt {
    SifrInt::from(items.len())
}

fn get_sum(items: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for item in items.iter().cloned() {
        total = &total + &item;
    }
    total.clone()
}

fn consume_and_reverse(items: Vec<SifrInt>) -> Vec<SifrInt> {
    Box::new((items).iter().cloned().rev()).collect::<Vec<_>>()
}

fn add_lengths(a: &Vec<SifrInt>, b: &Vec<SifrInt>) -> SifrInt {
    &SifrInt::from(a.len()) + &SifrInt::from(b.len())
}

fn double(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(2)
}

fn negate(x: f64) -> f64 {
    -x
}

fn main() {
    let data: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30), SifrInt::from_i64(40), SifrInt::from_i64(50)];
    let length: SifrInt = get_length(&data);
    let total: SifrInt = get_sum(&data);
    println!("{}", length);
    println!("{}", total);
    println!("{:?}", data);
    let items: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)];
    let result: Vec<SifrInt> = consume_and_reverse(items);
    println!("{:?}", result);
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)];
    let combined: SifrInt = add_lengths(&nums, &nums);
    println!("{}", combined);
    println!("{:?}", nums);
    let x: SifrInt = SifrInt::from_i64(42);
    let d: SifrInt = double((x).clone());
    println!("{}", d);
    println!("{}", x);
    let pi: f64 = 3.14_f64;
    let neg: f64 = negate(pi);
    println!("{}", neg);
    println!("{}", pi);
    let loop_data: Vec<SifrInt> = vec![SifrInt::from_i64(5), SifrInt::from_i64(10), SifrInt::from_i64(15)];
    let mut loop_total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from_i64(3), SifrInt::from_i64(1)) {
        loop_total = &loop_total + &get_sum(&loop_data);
    }
    println!("{}", loop_total);
    println!("{:?}", loop_data);
}
