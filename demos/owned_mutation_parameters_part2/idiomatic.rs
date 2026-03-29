fn mutate_and_return(mut items: Vec<i64>) -> Vec<i64> {
    items[0] = 9;
    items[1] = 10;
    items
}

fn mutate_borrowed(items: &mut [i64]) -> i64 {
    items[0] = 14;
    items.len() as i64
}

fn main() {
    let mut moved = mutate_and_return(vec![1_i64, 2, 3]);
    println!("{}", moved[0]);
    println!("{}", moved[1]);
    println!("{}", mutate_borrowed(&mut moved));
}
