// src/main.rs
fn payload_size(data: &Vec<i64>) -> i64 {
    data.len() as i64
}

fn main() {
    println!("{}", payload_size(&vec![1_i64, 2_i64, 3_i64]));
    println!("recursive alias names resolved");
}
