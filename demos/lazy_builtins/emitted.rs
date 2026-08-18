// src/main.rs
fn main() {
    let nums: Vec<i64> = vec![3_i64, 1_i64, 2_i64];
    let mut rev_it: Box<dyn Iterator<Item = i64>> = Box::new((nums).iter().copied().rev());
    println!("{}", (rev_it.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{:?}", rev_it.collect::<Vec<_>>());
    let indexed_it: Box<dyn Iterator<Item = (i64, String)>> = Box::new((vec!["a".to_string(), "b".to_string()]).into_iter().enumerate().map(|__pair| ((__pair.0 as i64) + (5_i64), __pair.1)));
    println!("{:?}", indexed_it.collect::<Vec<_>>());
    let zipped_it: Box<dyn Iterator<Item = (i64, String, bool)>> = Box::new((vec![1_i64, 2_i64]).into_iter().zip((vec!["x".to_string(), "y".to_string()]).into_iter()).zip((vec![true, false]).into_iter()).map(|__zip_item| (__zip_item.0.0, __zip_item.0.1, __zip_item.1)));
    println!("{:?}", zipped_it.collect::<Vec<_>>());
}
