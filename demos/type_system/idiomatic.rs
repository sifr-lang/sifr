#[derive(Debug, Clone)]
enum IntOrStr {
    Int(i64),
    Str(String),
}

fn create_user(id: i64, _name: &str) -> i64 {
    id
}

fn handle_command(cmd: &str) -> &'static str {
    match cmd {
        "start" => "Starting...",
        _ => "Unknown command",
    }
}

fn describe(x: &IntOrStr) -> String {
    match x {
        IntOrStr::Int(x) => format!("number: {}", x + 1),
        IntOrStr::Str(x) => format!("text: {}", x),
    }
}

fn find_user(name: &str) -> Option<&'static str> {
    if name == "alice" {
        Some("Alice Smith")
    } else {
        None
    }
}

fn main() {
    let uid = create_user(42, "alice");
    println!("{}", uid);
    println!("{}", handle_command("start"));
    println!("{}", handle_command("stop"));
    println!("{}", describe(&IntOrStr::Int(42)));
    println!("{}", describe(&IntOrStr::Str("hello".to_string())));
    let user = find_user("alice");
    if let Some(user) = user {
        println!("{}", user);
    } else {
        println!("not found");
    }
}
