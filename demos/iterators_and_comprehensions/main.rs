// Reference: generics
// Reference: generics
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/sifr emit demos/generics_demo.sifr`
fn main() {
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let doubled: Vec<i64> = nums.clone().into_iter().map(|x| x * 2_i64).collect::<Vec<_>>();
    println!("{:?}", doubled);
    let evens: Vec<i64> = nums.clone().into_iter().filter(|x| { let x = *x; (|x| x % 2_i64 == 0_i64)(x) }).collect::<Vec<_>>();
    println!("{:?}", evens);
    let squares: Vec<i64> = nums.clone().into_iter().map(|x| x * x).collect::<Vec<i64>>();
    println!("{:?}", squares);
    let big_squares: Vec<i64> = nums.iter().filter(|x| { let x = **x; x > 2_i64 }).map(|x| { let x = *x; x * x }).collect::<Vec<i64>>();
    println!("{:?}", big_squares);
    println!("{}", nums.iter().min().unwrap().clone());
    println!("{}", nums.iter().max().unwrap().clone());
    println!("{}", nums.iter().sum::<i64>());
    let unsorted: Vec<i64> = vec![5_i64, 3_i64, 1_i64, 4_i64, 2_i64];
    println!("{:?}", { let mut _sorted = unsorted.clone(); _sorted.sort(); _sorted });
    println!("{:?}", { let mut _rev = unsorted.clone(); _rev.reverse(); _rev });
    let letters: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("{:?}", letters.iter().enumerate().map(|(i, v)| (i as i64, v.clone())).collect::<Vec<_>>());
    let names: Vec<String> = vec!["Alice".to_string(), "Bob".to_string()];
    let ages: Vec<i64> = vec![30_i64, 25_i64];
    println!("{:?}", names.iter().zip(ages.iter()).map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>());
    let bools: Vec<bool> = vec![true, false, true];
    println!("{}", bools.iter().any(|x| *x));
    println!("{}", bools.iter().all(|x| *x));
    println!("{}", vec![true, true, true].iter().all(|x| *x));
}
