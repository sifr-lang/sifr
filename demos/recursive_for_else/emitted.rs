// src/main.rs
fn main() {
    fn rec(n: i64) -> i64 {
        let items: Vec<i64> = vec![1_i64];
        {
            let _broke: bool = false;
            for i in items.iter().copied() {
            }
            if !(_broke) {
                if n > (0_i64) {
                    return rec(n - (1_i64));
                }
            }
        }
        return 0_i64;
    }
    println!("recursive_for_else canonical walker coverage demo:");
    println!("{}", rec(3_i64));
}
