#[derive(Debug, Clone)]
enum IntOrStr {
    Int(i64),
    Str(String),
}

impl std::fmt::Display for IntOrStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntOrStr::Int(v) => {
                return write!(f, "{}", v);
            }
            IntOrStr::Str(v) => {
                return write!(f, "{}", v);
            }
        }
    }
}

fn create_user(id: i64, name: &String) -> i64 {
    return id;
}

fn handle_command(cmd: &String) -> String {
    if cmd.clone() == "start".to_string() {
        return "Starting...".to_string();
    } else {
        return "Unknown command".to_string();
    }
}

fn describe(x: &IntOrStr) -> String {
    if let IntOrStr::Int(x) = x {
        return format!("number: {}", x + (1 as i64));
    } else {
        if let IntOrStr::Str(x) = x {
            return format!("text: {}", x);
        } else {
            unreachable!("sifr union narrowing fell through exhaustive branch chain");
        }
    }
}

fn find_user(name: &String) -> Option<String> {
    if name.clone() == "alice".to_string() {
        return Some("Alice Smith".to_string());
    }
    return None;
}

fn main() {
    let uid: i64 = create_user(42 as i64, &"alice".to_string());
    println!("{}", uid);
    println!("{}", handle_command(&"start".to_string()));
    println!("{}", handle_command(&"stop".to_string()));
    println!("{}", describe(&IntOrStr::Int(42 as i64)));
    println!("{}", describe(&IntOrStr::Str("hello".to_string())));
    let user: Option<String> = find_user(&"alice".to_string());
    if let Some(user) = user {
        println!("{}", user);
    } else {
        println!("not found");
    }
}
