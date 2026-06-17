mod helper {
    pub fn adjusted(value: i64) -> i64 {
        value + 2
    }
}

fn main() {
    println!("project_entrypoint canonical frontend entry path demo:");
    println!("{}", helper::adjusted(5));
}
