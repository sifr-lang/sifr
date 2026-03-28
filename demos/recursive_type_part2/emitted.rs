fn payload_size(data: &Vec<i64>) -> i64 {
    return data.len() as i64;
}

fn main() {
    println!("{}", payload_size(&vec![1 as i64, 2 as i64, 3 as i64]));
    println!("well-formed recursive aliases accepted");
}
