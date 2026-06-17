// Reference: type_system
// Reference: compiler-feature-history
#[derive(Debug, Clone)]
enum IntOrStr {
    Int(i64),
    Str(String),
}

impl std::fmt::Display for IntOrStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntOrStr::Int(v) => write!(f, "{}", v),
            IntOrStr::Str(v) => write!(f, "{}", v),
        }
    }
}


fn create_user(id: i64, name: String) -> i64 {
    return id;
}

fn handle_command(cmd: String) -> String {
    if cmd == "start".to_string() {
        return "Starting...".to_string();
    } else {
        return "Unknown command".to_string();
    }
}

fn describe(x: IntOrStr) -> String {
    match x {
        IntOrStr::Int(x) => {
            return format!("number: {}", x + 1_i64);
        }
        IntOrStr::Str(x) => {
            return format!("text: {}", x);
        }
    }
}

fn find_user(name: String) -> Option<String> {
    if name == "alice".to_string() {
        return Some("Alice Smith".to_string());
    }
    return None;
}

fn main() {
    let uid: i64 = create_user(42_i64, "alice".to_string());
    println!("{}", uid);
    println!("{}", handle_command("start".to_string()));
    println!("{}", handle_command("stop".to_string()));
    println!("{}", describe(IntOrStr::Int(42_i64)));
    println!("{}", describe(IntOrStr::Str("hello".to_string())));
    let user: Option<String> = find_user("alice".to_string());
    if let Some(user) = user {
        println!("{}", user);
    } else {
        println!("{}", "not found");
    }
}
