enum IntOrStr {
    Int(i64),
    Str(String),
}

enum BoolIntOrStr {
    Bool(bool),
    Int(i64),
    Str(String),
}

fn find_user(name: &str) -> Option<&'static str> {
    if name == "alice" {
        Some("Alice Smith")
    } else {
        None
    }
}

fn process(x: &IntOrStr) -> String {
    match x {
        IntOrStr::Int(value) => format!("number: {value}"),
        IntOrStr::Str(value) => format!("string: {value}"),
    }
}

fn classify(x: &BoolIntOrStr) -> &'static str {
    match x {
        BoolIntOrStr::Int(value) => {
            let _non_negative = *value >= 0;
            "int"
        }
        BoolIntOrStr::Str(value) => {
            let _is_empty = value.is_empty();
            "str"
        }
        BoolIntOrStr::Bool(value) => {
            let _flag = *value;
            "bool"
        }
    }
}

fn process_optional(x: Option<&str>) -> String {
    match x {
        Some(value) => value.to_uppercase(),
        None => "none".to_string(),
    }
}

fn consume(s: String) -> String {
    s
}

fn main() {
    if let Some(result) = find_user("alice") {
        println!("{result}");
    }

    if find_user("bob").is_none() {
        println!("not found");
    }

    println!("{}", process(&IntOrStr::Int(42)));
    println!("{}", process(&IntOrStr::Str("hello".to_string())));
    println!("{}", classify(&BoolIntOrStr::Int(1)));
    println!("{}", classify(&BoolIntOrStr::Str("hi".to_string())));
    println!("{}", classify(&BoolIntOrStr::Bool(true)));
    println!("{}", process_optional(Some("world")));
    println!("{}", process_optional(None));

    let mut s = "hello".to_string();
    let x = consume(s);
    s = "world".to_string();
    println!("{s}");
    println!("{x}");
}
