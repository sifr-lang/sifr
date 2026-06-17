fn main() {
    fn recurse(n: i64) -> i64 {
        let mut _broke = false;
        for i in vec![1 as i64].into_iter() {
        }
        if !_broke {
            if n > (0 as i64) {
                return recurse(n - (1 as i64));
            }
        }
        return 0 as i64;
    }
    println!("recursive_calls semantic query layer standardization demo:");
    println!("{}", recurse(4 as i64));
}
