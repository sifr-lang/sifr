// src/main.rs
fn main() {
    fn recurse(n: i64) -> i64 {
        {
            let _broke: bool = false;
            for i in vec![1_i64].into_iter() {
            }
            if !(_broke) {
                if n > (0_i64) {
                    return recurse(n - (1_i64));
                }
            }
        }
        return 0_i64;
    }
    println!("recursive_calls semantic query layer standardization demo:");
    println!("{}", recurse(4_i64));
}
