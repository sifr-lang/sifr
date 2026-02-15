fn display(name: &String, verbose: bool) -> String {
    if verbose {
        return format!("Name: {}", name);
    }
    return name;
}

fn main() {
    println!("{}", display(&"Alice".to_string(), false));
    println!("{}", display(&"Bob".to_string(), true));
}
