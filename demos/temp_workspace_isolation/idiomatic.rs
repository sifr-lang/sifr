mod helper {
    pub fn value() -> i64 {
        44
    }
}

fn main() {
    println!("temp_workspace_isolation invocation-scoped temp workspace isolation demo:");
    println!("{}", helper::value());
}
