#[derive(Debug, Clone)]
enum BoolOrIntOrStr {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl std::fmt::Display for BoolOrIntOrStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolOrIntOrStr::Bool(v) => {
                return write!(f, "{}", v);
            }
            BoolOrIntOrStr::Int(v) => {
                return write!(f, "{}", v);
            }
            BoolOrIntOrStr::Str(v) => {
                return write!(f, "{}", v);
            }
        }
    }
}

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

fn find_user(name: &String) -> Option<String> {
    if name.clone() == "alice".to_string() {
        return Some("Alice Smith".to_string());
    }
    return None;
}

fn process(x: &IntOrStr) -> String {
    if let IntOrStr::Int(x) = x {
        return format!("number: {}", x);
    } else {
        if let IntOrStr::Str(x) = x {
            return format!("string: {}", x);
        } else {
            unreachable!("sifr union narrowing fell through exhaustive branch chain");
        }
    }
}

fn classify(x: &BoolOrIntOrStr) -> String {
    if let BoolOrIntOrStr::Int(x) = x {
        return "int".to_string();
    } else {
        if let BoolOrIntOrStr::Str(x) = x {
            return "str".to_string();
        } else {
            if let BoolOrIntOrStr::Bool(x) = x {
                return "bool".to_string();
            } else {
                unreachable!("sifr union narrowing fell through exhaustive branch chain");
            }
        }
    }
}

fn process_optional(x: &Option<String>) -> String {
    if let Some(x) = x.as_ref() {
        return x.to_uppercase();
    }
    return "none".to_string();
}

fn consume(s: String) -> String {
    return s;
}

fn main() {
    let result: Option<String> = find_user(&"alice".to_string());
    if let Some(result) = result {
        println!("{}", result);
    }
    let missing: Option<String> = find_user(&"bob".to_string());
    if missing.is_none() {
        println!("not found");
    }
    println!("{}", process(&IntOrStr::Int(42 as i64)));
    println!("{}", process(&IntOrStr::Str("hello".to_string())));
    println!("{}", classify(&BoolOrIntOrStr::Int(1 as i64)));
    println!("{}", classify(&BoolOrIntOrStr::Str("hi".to_string())));
    println!("{}", classify(&BoolOrIntOrStr::Bool(true)));
    println!("{}", process_optional(&Some("world".to_string())));
    println!("{}", process_optional(&None));
    let mut s: String = "hello".to_string();
    let x: String = consume(s);
    s = "world".to_string();
    println!("{}", s);
    println!("{}", x);
}
