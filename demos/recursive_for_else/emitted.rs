fn main() {
    fn rec(n: i64) -> i64 {
        let items: Vec<i64> = vec![1 as i64];
        let mut _broke = false;
        for i in items.iter().copied() {
        }
        if !_broke {
            if n > (0 as i64) {
                return rec(n - (1 as i64));
            }
        }
        return 0 as i64;
    }
    println!("recursive_for_else canonical walker coverage demo:");
    println!("{}", rec(3 as i64));
}
