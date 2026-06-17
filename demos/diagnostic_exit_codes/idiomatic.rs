mod helper {
    pub fn doubled(x: i64) -> i64 {
        x * 2
    }
}

fn main() {
    println!("diagnostic_exit_codes cross-mode diagnostic and exit behavior demo:");
    println!("{}", helper::doubled(21));
}
